check:
	echo "Just for checking out the repo"

build-cli:
	python3 ./scripts/build-cli.py

build-mgr:
	@python3 ./scripts/build-manager.py $@ 
