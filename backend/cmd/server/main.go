package main

import (
	"fmt"
	"log"
	"net/http"
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

const dbRetries = 30

func main() {
	cfg := config.Load()

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
