package main

import (
	"bufio"
	"database/sql"
	"log"
	"os"
	"strings"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/golang-jwt/jwt/v5"
	_ "github.com/lib/pq"
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

func getToken(c *fiber.Ctx) string {
	hdr := c.Get("Authorization")
	if hdr == "" {
		return ""
	}

	token := strings.Split(hdr, "Bearer ")[1]
	return token
}

func main() {
	app := fiber.New()
	db, err := sql.Open("postgres", "user=postgres password=123456 dbname=testbench sslmode=disable")

	if err != nil {
		return
	}

	defer db.Close()

	db.SetMaxOpenConns(10)
	db.SetMaxIdleConns(10)

	app.Get("/", func(c *fiber.Ctx) error {
		tokenString := getToken(c)
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

		query := "SELECT * FROM users WHERE email=$1"
		row := db.QueryRow(query, claims.Email)

		var user User = User{}
		err2 := row.Scan(&user.Email, &user.First, &user.Last, &user.County, &user.City, &user.Age)
		if err2 == sql.ErrNoRows {
			log.Println("ErrNoRows")
		}
		if err2 != nil {
			log.Println(err2)
			return c.SendStatus(fiber.StatusNotFound)
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

		rows, err := db.Query("SELECT * FROM USERS OFFSET floor(random() * 100000) LIMIT 10")
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
