package config

import (
	"fmt"
	"os"
)

type Config struct {
	DBHost     string
	DBPort     string
	DBUser     string
	DBPassword string
	DBName     string
	ServerPort string
	StaticDir  string
	JWTSecret  string
	WTPort     string
	WTHost     string
}

func Load() *Config {
	return &Config{
		DBHost:     getEnv("DB_HOST", "localhost"),
		DBPort:     getEnv("DB_PORT", "3306"),
		DBUser:     getEnv("DB_USER", "agon"),
		DBPassword: getEnv("DB_PASSWORD", "agon"),
		DBName:     getEnv("DB_NAME", "agon"),
		ServerPort: getEnv("SERVER_PORT", "7777"),
		StaticDir:  getEnv("STATIC_DIR", "./static"),
		JWTSecret:  getEnv("JWT_SECRET", "change-me-in-production"),
		WTPort:     getEnv("WT_PORT", "4433"),
		WTHost:     getEnv("WT_HOST", "localhost"),
	}
}

func (c *Config) DSN() string {
	return fmt.Sprintf("%s:%s@tcp(%s:%s)/%s?parseTime=true&charset=utf8mb4", c.DBUser, c.DBPassword, c.DBHost, c.DBPort, c.DBName)
}

func (c *Config) MigrateURL() string {
	return fmt.Sprintf("mysql://%s:%s@tcp(%s:%s)/%s?multiStatements=true", c.DBUser, c.DBPassword, c.DBHost, c.DBPort, c.DBName)
}

func getEnv(key, defaultVal string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return defaultVal
}
