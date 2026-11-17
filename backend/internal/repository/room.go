package repository

import (
	"github.com/yumikokawaii/agon/internal/model"
	"gorm.io/gorm"
)

type RoomRepository interface {
	Create(r *model.Room) error
	FindByID(id string) (*model.Room, error)
	Join(id string, guestID int64) error
	SetReady(id string, seat int, ready bool) error
	StartIfReady(id, matchID string) (bool, error)
	RemoveGuest(id string) error
	Close(id string) error
	EndMatch(id, matchID string) error
	ResetPlaying() error
}

type roomRepo struct{ db *gorm.DB }

func NewRoomRepo(db *gorm.DB) RoomRepository { return &roomRepo{db} }

func (r *roomRepo) Create(room *model.Room) error { return translate(r.db.Create(room).Error) }

func (r *roomRepo) FindByID(id string) (*model.Room, error) {
	var room model.Room
	if err := r.db.Preload("Host").Preload("Guest").First(&room, "id = ?", id).Error; err != nil {
		return nil, translate(err)
	}
	return &room, nil
}

func (r *roomRepo) Join(id string, guestID int64) error {
	return r.db.Model(&model.Room{}).
		Where("id = ? AND host_id <> ? AND guest_id IS NULL AND state = ?", id, guestID, model.RoomWaiting).
		Update("guest_id", guestID).Error
}

func (r *roomRepo) SetReady(id string, seat int, ready bool) error {
	col := "host_ready"
	if seat == 1 {
		col = "guest_ready"
	}
	return r.db.Model(&model.Room{}).Where("id = ? AND state = ?", id, model.RoomWaiting).Update(col, ready).Error
}

func (r *roomRepo) StartIfReady(id, matchID string) (bool, error) {
	res := r.db.Model(&model.Room{}).
		Where("id = ? AND state = ? AND host_ready AND guest_ready AND guest_id IS NOT NULL", id, model.RoomWaiting).
		Updates(map[string]any{"state": model.RoomPlaying, "match_id": matchID})
	return res.RowsAffected == 1, res.Error
}

func (r *roomRepo) RemoveGuest(id string) error {
	return r.db.Model(&model.Room{}).Where("id = ?", id).Updates(map[string]any{
		"guest_id": nil, "host_ready": false, "guest_ready": false, "state": model.RoomWaiting, "match_id": "",
	}).Error
}

func (r *roomRepo) Close(id string) error {
	return r.db.Model(&model.Room{}).Where("id = ?", id).Update("state", model.RoomClosed).Error
}

func (r *roomRepo) ResetPlaying() error {
	return r.db.Model(&model.Room{}).Where("state = ?", model.RoomPlaying).Updates(map[string]any{
		"host_ready": false, "guest_ready": false, "state": model.RoomWaiting, "match_id": "",
	}).Error
}

func (r *roomRepo) EndMatch(id, matchID string) error {
	return r.db.Model(&model.Room{}).Where("id = ? AND match_id = ? AND state = ?", id, matchID, model.RoomPlaying).Updates(map[string]any{
		"host_ready": false, "guest_ready": false, "state": model.RoomWaiting, "match_id": "",
	}).Error
}
