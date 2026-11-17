package repository

import (
	"github.com/yumikokawaii/agon/internal/model"
	"gorm.io/gorm"
)

type UserRepository interface {
	Create(u *model.User) error
	FindByID(id int64) (*model.User, error)
	FindByName(name string) (*model.User, error)
	UpdateKit(id int64, kit int32) error
	BumpTokenVersion(id int64) error
}

type userRepo struct{ db *gorm.DB }

func NewUserRepo(db *gorm.DB) UserRepository { return &userRepo{db} }

func (r *userRepo) Create(u *model.User) error { return translate(r.db.Create(u).Error) }

func (r *userRepo) FindByID(id int64) (*model.User, error) {
	var u model.User
	if err := r.db.First(&u, id).Error; err != nil {
		return nil, translate(err)
	}
	return &u, nil
}

func (r *userRepo) FindByName(name string) (*model.User, error) {
	var u model.User
	if err := r.db.Where("lower(name) = lower(?)", name).First(&u).Error; err != nil {
		return nil, translate(err)
	}
	return &u, nil
}

func (r *userRepo) UpdateKit(id int64, kit int32) error {
	return r.db.Model(&model.User{}).Where("id = ?", id).Update("kit", kit).Error
}

func (r *userRepo) BumpTokenVersion(id int64) error {
	return r.db.Model(&model.User{}).Where("id = ?", id).Update("token_version", gorm.Expr("token_version + 1")).Error
}
