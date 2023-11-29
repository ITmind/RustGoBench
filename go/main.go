package main

import (
	"bufio"
	"context"
	"database/sql"
	"log"
	"os"
	"strings"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/golang-jwt/jwt/v5"
	"github.com/jackc/pgx/v5/pgxpool"
)

type MyCustomClaims struct {
	Email string `json:"email"`
	jwt.RegisteredClaims
}

type User struct {
	Email  string
	First  string
	Last   string
	City   string
	County string
	Age    int
}

var jwtSecret = "mysuperPUPERsecret100500security"

func main() {
	app := fiber.New()
	db, err := pgxpool.New(context.Background(), `user=postgres password=123456 dbname=testbench 
			sslmode=disable pool_max_conns=10 pool_min_conns=10`)

	if err != nil {
		return
	}

	defer db.Close()

	app.Get("/", func(c *fiber.Ctx) error {

		authHeader := c.Get("Authorization")
		if authHeader == "" {
			return c.SendStatus(fiber.StatusUnauthorized)
		}

		tokenString := strings.Split(authHeader, "Bearer ")[1]

		if tokenString == "" {
			return c.SendStatus(fiber.StatusUnauthorized)
		}
		token, err := jwt.ParseWithClaims(tokenString, &MyCustomClaims{}, func(token *jwt.Token) (interface{}, error) {
			return []byte(jwtSecret), nil
		})

		if err != nil {
			log.Println(err)
			return c.SendStatus(fiber.StatusUnauthorized)
		}

		claims := token.Claims.(*MyCustomClaims)
		row := db.QueryRow(c.Context(), "SELECT * FROM users WHERE email=$1", claims.Email)

		var user User = User{}
		err = row.Scan(&user.Email, &user.First, &user.Last, &user.County, &user.City, &user.Age)
		if err == sql.ErrNoRows {
			return c.SendStatus(fiber.StatusNotFound)
		}
		if err != nil {
			log.Println(err)
			return c.SendStatus(fiber.StatusInternalServerError)
		}

		return c.JSON(user)
	})

	//вспомогательная ручка
	app.Get("/randomtoken", func(c *fiber.Ctx) error {
		file, err := os.Create("tokens.txt")
		if err != nil {
			log.Println(err)
			return c.SendStatus(fiber.StatusInternalServerError)
		}

		writer := bufio.NewWriter(file)

		rows, err := db.Query(c.Context(), "SELECT * FROM USERS OFFSET floor(random() * 100000) LIMIT 10")
		if err != nil {
			return c.SendStatus(fiber.StatusInternalServerError)
		}

		for rows.Next() {
			var user User
			err = rows.Scan(&user.Email, &user.First, &user.Last, &user.County, &user.City, &user.Age)
			if err != nil {
				log.Println(err)
				return c.SendStatus(fiber.StatusInternalServerError)
			}

			claims := MyCustomClaims{
				user.Email,
				jwt.RegisteredClaims{
					ExpiresAt: jwt.NewNumericDate(time.Now().Add(24 * time.Hour)),
				},
			}

			token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
			ss, err := token.SignedString([]byte(jwtSecret))

			if err != nil {
				log.Println(err)
				return c.SendStatus(fiber.StatusInternalServerError)
			}

			_, err = writer.WriteString(ss + "\n")
			if err != nil {
				file.Close()
				log.Println(err)
				return c.SendStatus(fiber.StatusInternalServerError)
			}

		}

		writer.Flush()
		file.Close()

		return c.SendFile(file.Name())
	})

	log.Fatal(app.Listen(":3000"))
}
