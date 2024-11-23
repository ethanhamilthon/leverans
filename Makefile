check:
	echo "Just for checking out the repo"

build-cli:
	python3 ./scripts/build-cli.py

build-mgr:
	python3 ./scripts/build-manager.py $(V) 

release:
	python3 ./scripts/cli-release.py 

launch:
	./scripts/run.sh $(V) 

clean:
	./scripts/clean.sh
