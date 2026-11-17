package config

import (
	"net"
	"net/url"
	"os"
)

type Config struct {
	DBHost     string
	DBPort     string
	DBUser     string
	DBPassword string
	DBName     string
	DBSSLMode  string
	ServerPort string
	StaticDir  string
	JWTSecret  string
	WTPort     string
	WTHost     string
}

func Load() *Config {
	return &Config{
		DBHost:     getEnv("DB_HOST", "localhost"),
		DBPort:     getEnv("DB_PORT", "5432"),
		DBUser:     getEnv("DB_USER", "agon"),
		DBPassword: getEnv("DB_PASSWORD", "agon"),
		DBName:     getEnv("DB_NAME", "agon"),
		DBSSLMode:  getEnv("DB_SSLMODE", "disable"),
		ServerPort: getEnv("SERVER_PORT", "7777"),
		StaticDir:  getEnv("STATIC_DIR", "./static"),
		JWTSecret:  os.Getenv("JWT_SECRET"),
		WTPort:     getEnv("WT_PORT", "4433"),
		WTHost:     getEnv("WT_HOST", "localhost"),
	}
}

func (c *Config) url(scheme string) string {
	u := url.URL{
		Scheme:   scheme,
		User:     url.UserPassword(c.DBUser, c.DBPassword),
		Host:     net.JoinHostPort(c.DBHost, c.DBPort),
		Path:     "/" + c.DBName,
		RawQuery: url.Values{"sslmode": {c.DBSSLMode}}.Encode(),
	}
	return u.String()
}

func (c *Config) DSN() string {
	return c.url("postgres")
}

func (c *Config) MigrateURL() string {
	return c.url("pgx5")
}

func getEnv(key, defaultVal string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return defaultVal
}
