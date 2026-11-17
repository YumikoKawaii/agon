package main

import (
	"fmt"
	"log"
	"net/http"
	"strings"
	"time"

	"connectrpc.com/connect"
	"github.com/golang-migrate/migrate/v4"
	_ "github.com/golang-migrate/migrate/v4/database/mysql"
	_ "github.com/golang-migrate/migrate/v4/source/file"
	"gorm.io/driver/mysql"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"

	"github.com/yumikokawaii/agon/gen/agon/v1/agonv1connect"
	"github.com/yumikokawaii/agon/internal/arena"
	"github.com/yumikokawaii/agon/internal/config"
	"github.com/yumikokawaii/agon/internal/repository"
	"github.com/yumikokawaii/agon/internal/rpchandler"
	"github.com/yumikokawaii/agon/internal/service"
	"github.com/yumikokawaii/agon/internal/uow"
)

const (
	dbRetries    = 30
	minSecretLen = 32
)

func main() {
	cfg := config.Load()
	if len(cfg.JWTSecret) < minSecretLen {
		log.Fatalf("JWT_SECRET must be set to at least %d characters", minSecretLen)
	}

	if err := runMigrations(cfg); err != nil {
		log.Fatalf("migration failed: %v", err)
	}

	db, err := gorm.Open(mysql.Open(cfg.DSN()), &gorm.Config{
		Logger:         logger.Default.LogMode(logger.Warn),
		TranslateError: true,
	})
	if err != nil {
		log.Fatalf("failed to connect database: %v", err)
	}

	unitOfWork := uow.New(db)
	userRepo := repository.NewUserRepo(db)

	matchArena, err := arena.New(cfg.WTPort, cfg.WTHost)
	if err != nil {
		log.Fatalf("arena: %v", err)
	}

	hub := service.NewRoomHub()
	authSvc := service.NewAuthService(userRepo, cfg.JWTSecret)
	userSvc := service.NewUserService(userRepo)
	roomSvc := service.NewRoomService(unitOfWork, hub, matchArena)
	matchSvc := service.NewMatchService(unitOfWork, hub, matchArena)
	if err := matchSvc.AbortStale(); err != nil {
		log.Fatalf("abort stale matches: %v", err)
	}

	go func() {
		for r := range matchArena.Results() {
			if err := matchSvc.Finish(r.MatchID, r.WinnerSeat, r.Deaths); err != nil {
				log.Printf("match %s: finish: %v", r.MatchID, err)
			}
		}
	}()
	go func() {
		log.Printf("webtransport listening on udp :%s", cfg.WTPort)
		log.Fatalf("arena failed: %v", matchArena.Serve())
	}()

	interceptor := connect.WithInterceptors(rpchandler.NewAuthenticator(authSvc))

	mux := http.NewServeMux()
	mux.Handle(agonv1connect.NewAuthServiceHandler(rpchandler.NewAuthServiceHandler(authSvc), interceptor))
	mux.Handle(agonv1connect.NewUserServiceHandler(rpchandler.NewUserServiceHandler(userSvc), interceptor))
	mux.Handle(agonv1connect.NewRoomServiceHandler(rpchandler.NewRoomServiceHandler(roomSvc), interceptor))
	mux.Handle(agonv1connect.NewMatchServiceHandler(rpchandler.NewMatchServiceHandler(matchSvc), interceptor))
	mux.Handle("/", static(cfg.StaticDir))

	addr := fmt.Sprintf(":%s", cfg.ServerPort)
	log.Printf("server listening on %s", addr)
	var protocols http.Protocols
	protocols.SetHTTP1(true)
	protocols.SetUnencryptedHTTP2(true)
	srv := &http.Server{Addr: addr, Handler: mux, Protocols: &protocols}
	if err := srv.ListenAndServe(); err != nil {
		log.Fatalf("server failed: %v", err)
	}
}

func static(dir string) http.Handler {
	files := http.FileServer(http.Dir(dir))
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if strings.HasSuffix(r.URL.Path, ".wasm") {
			w.Header().Set("Content-Type", "application/wasm")
		}
		w.Header().Set("Cache-Control", "no-cache")
		files.ServeHTTP(w, r)
	})
}

func runMigrations(cfg *config.Config) error {
	var m *migrate.Migrate
	var err error
	for i := 0; i < dbRetries; i++ {
		if m, err = migrate.New("file://db/migrations", cfg.MigrateURL()); err == nil {
			break
		}
		time.Sleep(time.Second)
	}
	if err != nil {
		return err
	}
	defer m.Close()
	if err := m.Up(); err != nil && err != migrate.ErrNoChange {
		return err
	}
	log.Println("migrations applied")
	return nil
}
