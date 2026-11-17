package repository

import (
	"time"

	"github.com/yumikokawaii/agon/internal/model"
	"gorm.io/gorm"
)

type MatchRepository interface {
	Create(m *model.Match) error
	FindByID(id string) (*model.Match, error)
	Finish(id string, winnerID *int64, deaths [2]int32) error
	AbortOpen() error
}

type matchRepo struct{ db *gorm.DB }

func NewMatchRepo(db *gorm.DB) MatchRepository { return &matchRepo{db} }

func (r *matchRepo) Create(m *model.Match) error { return translate(r.db.Create(m).Error) }

func (r *matchRepo) FindByID(id string) (*model.Match, error) {
	var m model.Match
	if err := r.db.Preload("Winner").First(&m, "id = ?", id).Error; err != nil {
		return nil, translate(err)
	}
	return &m, nil
}

func (r *matchRepo) AbortOpen() error {
	return r.db.Model(&model.Match{}).Where("ended_at IS NULL").Update("ended_at", time.Now()).Error
}

func (r *matchRepo) Finish(id string, winnerID *int64, deaths [2]int32) error {
	return r.db.Model(&model.Match{}).Where("id = ? AND ended_at IS NULL", id).Updates(map[string]any{
		"winner_id": winnerID, "host_deaths": deaths[0], "guest_deaths": deaths[1], "ended_at": time.Now(),
	}).Error
}
