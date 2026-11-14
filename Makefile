build-web:
	cd ./web && rm -rf node_modules && yarn install && yarn build

copy-web: build-web
	rm -rf backend/public/*
	cp ./web/dist/* backend/public

build-front: build-web copy-web

package: build-web copy-web
	docker build --platform linux/amd64 -f backend/Dockerfile -t tacogips/mongo-app-on-container-example:latest backend
	docker push tacogips/mongo-app-on-container-example:latest

