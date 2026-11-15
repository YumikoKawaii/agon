package service

import (
	"errors"
	"regexp"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/yumikokawaii/agon/internal/model"
	"github.com/yumikokawaii/agon/internal/repository"
	"golang.org/x/crypto/bcrypt"
)

const tokenTTL = 30 * 24 * time.Hour

var nameRe = regexp.MustCompile(`^[A-Za-z0-9_-]{3,16}$`)

type JWTClaims struct {
	UserID int64 `json:"uid"`
	jwt.RegisteredClaims
}

type AuthService struct {
	users     repository.UserRepository
	jwtSecret []byte
}

func NewAuthService(users repository.UserRepository, jwtSecret string) *AuthService {
	return &AuthService{users: users, jwtSecret: []byte(jwtSecret)}
}

func (s *AuthService) Register(name, password string) (*model.User, string, error) {
	if !nameRe.MatchString(name) {
		return nil, "", fail(ErrBadRequest, "username: 3-16 letters, digits, _ or -")
	}
	if len(password) < 6 || len(password) > 72 {
		return nil, "", fail(ErrBadRequest, "password: 6-72 characters")
	}
	hash, err := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
	if err != nil {
		return nil, "", err
	}
	u := &model.User{Name: name, PasswordHash: string(hash)}
	err = s.users.Create(u)
	if errors.Is(err, repository.ErrConflict) {
		return nil, "", fail(ErrConflict, "username taken")
	}
	if err != nil {
		return nil, "", err
	}
	return s.issue(u)
}

func (s *AuthService) Login(name, password string) (*model.User, string, error) {
	u, err := s.users.FindByName(name)
	if errors.Is(err, repository.ErrNotFound) {
		return nil, "", fail(ErrUnauthenticated, "wrong username or password")
	}
	if err != nil {
		return nil, "", err
	}
	if bcrypt.CompareHashAndPassword([]byte(u.PasswordHash), []byte(password)) != nil {
		return nil, "", fail(ErrUnauthenticated, "wrong username or password")
	}
	return s.issue(u)
}

func (s *AuthService) issue(u *model.User) (*model.User, string, error) {
	now := time.Now()
	claims := JWTClaims{
		UserID: u.ID,
		RegisteredClaims: jwt.RegisteredClaims{
			ExpiresAt: jwt.NewNumericDate(now.Add(tokenTTL)),
			IssuedAt:  jwt.NewNumericDate(now),
		},
	}
	token, err := jwt.NewWithClaims(jwt.SigningMethodHS256, claims).SignedString(s.jwtSecret)
	return u, token, err
}

func (s *AuthService) ParseJWT(token string) (*JWTClaims, error) {
	t, err := jwt.ParseWithClaims(token, &JWTClaims{}, func(t *jwt.Token) (any, error) {
		if _, ok := t.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, ErrUnauthenticated
		}
		return s.jwtSecret, nil
	})
	if err != nil {
		return nil, err
	}
	claims, ok := t.Claims.(*JWTClaims)
	if !ok || !t.Valid {
		return nil, ErrUnauthenticated
	}
	return claims, nil
}
