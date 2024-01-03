include .env.local
export

.PHONY:	 test

check:
	cargo clippy

run:
	cargo run

create-test:
	curl -v -H "Authorization: ${AUTHZ}" -H "Content-Type: application/json" "http://localhost:3030/q" -d '{"title": "this is shitty content", "content": "Test cnt", "tags": ["testing", "misc."]}'

update-test:
	curl -XPUT -H "Content-Type: application/json" "http://localhost:3030/q/10" -d '{"title": "Update test 10", "content": "Test cnt", "tags": ["updd"]}'

list:
	curl -H "id: id-list" "localhost:3030/q"

del:
	curl -XDELETE "localhost:3030/q/${ID}"

ans:
	curl -XPOST -H "Content-Type: application/x-www-form-urlencoded" "localhost:3030/a" --data-urlencode "qid=${QID}" --data-urlencode "content=The answer"

reg:
	curl -XPOST -H "Content-Type: application/json" "http://localhost:3030/reg" -d '{"email": "foo@bar", "password": "foobar"}'

log:
	curl -v -XPOST -H "Content-Type: application/json" "http://localhost:3030/login" -d '{"email": "foo@bar", "password": "foobar"}'

migrate:
	sqlx migrate run --database-url postgresql://dev:dev@localhost:5432/rustwebdev

rev-migrate:
	sqlx migrate revert --database-url postgresql://dev:dev@localhost:5432/rustwebdev

new-migrate:
	sqlx migrate add -r ${TBL}

apilayer:
	curl -v -XPOST 'https://api.apilayer.com/bad_words?censor_character=*' \
		# -H 'apikey: ${API_LAYER_K}' \
		-d 'shit length'

docker_u2004:
	docker build -t hellorust -f docker/ubuntu_20_04.Dockerfile .

docker_musl:
	docker build -t hellorust -f docker/musl.Dockerfile .
