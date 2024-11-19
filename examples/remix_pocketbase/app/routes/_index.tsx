import type { ActionFunctionArgs, MetaFunction } from "@remix-run/node";
import { Form, redirect, useLoaderData } from "@remix-run/react";
import PocketBase from "pocketbase";
const pbUrl = "http://" + (process.env.PB_URL || "");
const pb = new PocketBase(pbUrl);

export const meta: MetaFunction = () => {
  return [
    { title: "Leverans Remix app" },
    { name: "description", content: "Welcome to Lev!" },
  ];
};

type Todo = {
  id: string;
  title: string;
  desc: boolean;
};

export async function loader() {
  console.log("loader");
  const records = await pb.collection("posts").getFullList();

  const content = records.map((record) => ({
    id: record.id,
    title: record.title,
    desc: record.desc,
  }));
  return content as Todo[];
}

export async function action({ request }: ActionFunctionArgs) {
  console.log("action");
  const formData = await request.formData();
  await pb.collection("posts").create({
    title: formData.get("title") as string,
    desc: formData.get("desc") as string,
  });
  return redirect("/");
}

export default function Index() {
  const todos = useLoaderData<Todo[]>();
  return (
    <div className="flex justify-center w-full min-h-screen items-center">
      <div className="p-4 bg-neutral-800 rounded-md flex flex-col gap-4">
        <h1 className="text-3xl">
          Remix + PocketBase + Leverans example project
        </h1>
        <div className="flex flex-col w-96 gap-4">
          {todos.map((todo) => (
            <div
              key={todo.id}
              className="w-full p-2 bg-neutral-700/50 rounded-sm"
            >
              <p className="text-2xl">{todo.title}</p>
              <p>{todo.desc}</p>
            </div>
          ))}
        </div>
        <Form method="post" action="/?index">
          <input
            type="text"
            name="title"
            placeholder="Title"
            className="w-full p-2 bg-neutral-700/50 rounded-sm"
          />
          <input
            type="text"
            name="desc"
            placeholder="Description"
            className="w-full p-2 bg-neutral-700/50 rounded-sm"
          />
          <button
            type="submit"
            className="w-full p-2 bg-neutral-600/50 rounded-sm"
          >
            Create
          </button>
        </Form>
      </div>
    </div>
  );
}
