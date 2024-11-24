use std::{
    borrow::Borrow,
    sync::{Arc, LazyLock, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use actix_web::{
    error::InternalError, http::StatusCode, web, HttpRequest, HttpResponse, Responder, Result,
};
use anyhow::{anyhow, Result as AnyResult};
use bcrypt::verify;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use shared::{ok, UserAuthBody, UserSafe};

use crate::repo::user_repo::{RoleType, User};

use super::ServerData;

pub async fn handle_is_super_user_exists(
    sd: web::Data<Arc<ServerData>>,
    req: HttpRequest,
) -> Result<impl Responder> {
    must_have_levpass(&req)?;
    let pool = sd.repo.pool.borrow();
    match User::super_user_exists(pool).await.map_err(|e| {
        dbg!(e);
        InternalError::new(
            "Failed to check if super user exists",
            StatusCode::from_u16(500).unwrap(),
        )
    })? {
        true => ok!(HttpResponse::Ok().body("Super user exists")),
        false => ok!(HttpResponse::NotFound().body("Super user does not exist")),
    }
}

pub async fn login_user(
    sd: web::Data<Arc<ServerData>>,
    body: web::Json<UserAuthBody>,
    req: HttpRequest,
) -> Result<impl Responder> {
    must_have_levpass(&req)?;
    let pool = sd.repo.pool.borrow();
    let user = User::get_by_username(&body.username, pool)
        .await
        .map_err(|_| {
            InternalError::new(
                "Failed to get user, maybe there is no user with that name",
                StatusCode::from_u16(500).unwrap(),
            )
        })?;
    if !verify(body.password.clone(), &user.password_hash)
        .map_err(|_| InternalError::new("Wrong password", StatusCode::from_u16(401).unwrap()))?
    {
        return Err(
            InternalError::new("Wrong password", StatusCode::from_u16(401).unwrap()).into(),
        );
    }
    ok!(
        HttpResponse::Ok().body(create_jwt(body.username.borrow(), user.role).map_err(|_| {
            InternalError::new("Failed to login user", StatusCode::from_u16(500).unwrap())
        })?)
    )
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserBody {
    username: String,
    password: String,
    role: String,
}

impl CreateUserBody {
    pub fn get_role(&self) -> RoleType {
        match self.role.as_str() {
            "1" => RoleType::FullAccess,
            "2" => RoleType::UpdateOnly,
            "3" => RoleType::ReadOnly,
            _ => RoleType::ReadOnly,
        }
    }
}

pub async fn user_list(sd: web::Data<Arc<ServerData>>, req: HttpRequest) -> Result<impl Responder> {
    must_auth(&req, vec![RoleType::SuperUser])?;
    let pool = sd.repo.pool.borrow();
    let user_list: Vec<UserSafe> = User::get_all(pool)
        .await
        .map_err(|_| {
            InternalError::new(
                "Failed to get user list",
                StatusCode::from_u16(500).unwrap(),
            )
        })?
        .into_iter()
        .map(|u| UserSafe {
            username: u.username,
            role: u.role.to_string(),
        })
        .collect();
    ok!(web::Json(user_list))
}

pub async fn create_new_user(
    sd: web::Data<Arc<ServerData>>,
    body: web::Json<CreateUserBody>,
    req: HttpRequest,
) -> Result<impl Responder> {
    must_auth(&req, vec![RoleType::SuperUser])?;
    let pool = sd.repo.pool.borrow();
    User::new(
        body.username.clone(),
        body.password.clone(),
        body.get_role().to_string().as_str(),
    )
    .map_err(|_| {
        InternalError::new(
            "Failed to create super user",
            StatusCode::from_u16(500).unwrap(),
        )
    })?
    .insert_db(pool)
    .await
    .map_err(|_| {
        InternalError::new(
            "Failed to create super user",
            StatusCode::from_u16(500).unwrap(),
        )
    })?;
    ok!(HttpResponse::Ok().body(format!(
        "User {} created with right {}",
        body.username,
        body.get_role().to_string()
    )))
}

pub async fn register_super_user(
    sd: web::Data<Arc<ServerData>>,
    body: web::Json<UserAuthBody>,
    req: HttpRequest,
) -> Result<impl Responder> {
    must_have_levpass(&req)?;
    let pool = sd.repo.pool.borrow();
    User::new(
        body.username.clone(),
        body.password.clone(),
        RoleType::SuperUser.to_string().as_str(),
    )
    .map_err(|_| {
        InternalError::new(
            "Failed to create super user",
            StatusCode::from_u16(500).unwrap(),
        )
    })?
    .insert_db(pool)
    .await
    .map_err(|_| {
        InternalError::new(
            "Failed to create super user",
            StatusCode::from_u16(500).unwrap(),
        )
    })?;
    ok!(HttpResponse::Ok().body(
        create_jwt(body.username.borrow(), RoleType::SuperUser).map_err(|_| {
            InternalError::new(
                "Failed to create super user",
                StatusCode::from_u16(500).unwrap(),
            )
        })?
    ))
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: usize,
    role: String,
}

pub static JWT_KEY: LazyLock<Mutex<String>> =
    LazyLock::new(|| Mutex::new("dont_even_try_to_hack_me!".to_string()));

pub fn change_jwt_secret(secret_key: &str) {
    *JWT_KEY.lock().unwrap() = secret_key.to_string();
}

pub fn create_jwt(username: &str, role: RoleType) -> AnyResult<String> {
    let claims = Claims {
        sub: username.to_string(),
        exp: (SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 60 * 60 * 1000) as usize,
        role: role.to_string(),
    };

    let header = Header::default();
    let secret_key = JWT_KEY.lock().unwrap().clone();
    let key = EncodingKey::from_secret(secret_key.as_ref());

    let token = encode(&header, &claims, &key)?;
    Ok(token)
}

pub fn verify_jwt(token: &str) -> AnyResult<Claims> {
    let secret_key = JWT_KEY.lock().unwrap().clone();
    let key = DecodingKey::from_secret(secret_key.as_ref());
    let mut validation = Validation::default();
    validation.validate_exp = false;

    let token_data = decode::<Claims>(token, &key, &validation)?;
    Ok(token_data.claims)
}

pub fn check_header(req: &HttpRequest) -> AnyResult<()> {
    let token = req
        .headers()
        .get("X-LEVERANS-PASS")
        .ok_or(anyhow!("could not find X-LEVERANS-PASS"))?
        .to_str()?;
    println!("pass token: {}", token);
    if token != "true" {
        return Err(anyhow!("could not find X-LEVERANS-PASS"));
    }
    Ok(())
}

pub fn must_have_levpass(req: &HttpRequest) -> Result<()> {
    match check_header(req)
        .map_err(|_| InternalError::new("Forbidden", StatusCode::from_u16(403).unwrap()))
    {
        Ok(e) => Ok(e),
        Err(e) => Err(e.into()),
    }
}

pub fn must_auth(req: &HttpRequest, should_be: Vec<RoleType>) -> Result<String> {
    match check_auth(req).map_err(|e| {
        println!("error: {:?}", e);
        InternalError::new("Unauthorized", StatusCode::from_u16(401).unwrap())
    }) {
        Ok(e) => {
            if should_be.is_empty() {
                return Ok(e.sub);
            }
            if !should_be.contains(&RoleType::from_string(&e.role)) {
                return Err(InternalError::new(
                    format!(
                        "Forbidden: role should be: {} ",
                        should_be.get(0).unwrap().to_string()
                    ),
                    StatusCode::from_u16(403).unwrap(),
                )
                .into());
            }
            Ok(e.sub)
        }
        Err(e) => Err(e.into()),
    }
}

pub fn check_auth(req: &HttpRequest) -> AnyResult<Claims> {
    check_header(req)?;
    let token = req
        .headers()
        .get("Authorization")
        .ok_or(anyhow!("could not find Authorization"))?
        .to_str()?;
    let claims = verify_jwt(token)?;
    ok!(claims)
}
