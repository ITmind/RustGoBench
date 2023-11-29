# Сравнение производительности web-сервисов на Go и Rust

Для Go используем Fiber.  
Для Rust используем Axum.  
В качестве БД используем PostgreSQL

1. ### Подготавливаем БД  
Устанавливаем [Synth](https://www.getsynth.com)  
В терминале заходим в директорю data и выполняем:  
>_$ docker-compose up_  
_$ synth generate ./ --to postgres://postgres:123456@localhost:5432/testbench --size 100000_

2. ### Собираем web-сервис на Go перейдя в папку Go и выполнив  
>_$ go build_  

Заходим по адресу http://127.0.0.1:3000/randomtoken и копируем любой токен

3. ### Собираем web-сервис на Go перейдя в папку Go и выполнив  
>_$ cargo build --release_

4. ### Скачиваем [Cassowary](https://github.com/rogerwelin/cassowary) и запускаем его подставив скопированный ранее токен  
>_$ cassowary.exe run -u http://127.0.0.1:3000 -c 10 -n 100000 -H "Authorization:Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJlbWFpbCI6ImFsZWphbmRyaW5fZG9sb3JlbXF1ZUBnbWFpbC5jb20iLCJleHAiOjE3MDEzMDYyMTl9.5mT3KVV9Q69yd5gx-z97LVr6tgNA1yVJxpeJEXSq6U0"_
