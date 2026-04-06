package repository

import (
	"context"
	"errors"
	"strings"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/jackc/pgx/v5/pgxpool"
)

var (
	ErrNotFound      = errors.New("not found")
	ErrAlreadyExists = errors.New("already exists")
)

type Tag struct {
	TagID     string
	Name      string
	UserID    string
	CreatedAt int64
}

type NoteSummary struct {
	NoteID          string
	Title           string
	ContentPreview  string
	UserID          string
	CreatedAt       int64
	UpdatedAt       int64
}

type NoteDetail struct {
	NoteID    string
	Title     string
	Content   string
	UserID    string
	CreatedAt int64
	UpdatedAt int64
	Tags      []Tag
}

type NoteRepository interface {
	CreateNote(ctx context.Context, userID, title, content string) (NoteDetail, error)
	GetNote(ctx context.Context, userID, noteID string) (NoteDetail, error)
	ListNotes(ctx context.Context, userID string) ([]NoteSummary, error)
	SearchNotes(ctx context.Context, userID, query string) ([]NoteSummary, error)
	CreateTag(ctx context.Context, userID, name string) (Tag, error)
	ListTags(ctx context.Context, userID string) ([]Tag, error)
	AttachTagToNote(ctx context.Context, userID, noteID, tagID string) (NoteDetail, error)
}

type PostgresNoteRepository struct {
	pool *pgxpool.Pool
}

func NewPostgresNoteRepository(pool *pgxpool.Pool) *PostgresNoteRepository {
	return &PostgresNoteRepository{pool: pool}
}

func (r *PostgresNoteRepository) CreateNote(ctx context.Context, userID, title, content string) (NoteDetail, error) {
	_, err := r.pool.Exec(
		ctx,
		"INSERT INTO notes (title, content, user_id) VALUES ($1, $2, $3::int)",
		title,
		content,
		userID,
	)
	if err != nil {
		return NoteDetail{}, err
	}

	var noteID string
	err = r.pool.QueryRow(
		ctx,
		"SELECT CAST(id AS TEXT) FROM notes WHERE user_id = $1::int ORDER BY id DESC LIMIT 1",
		userID,
	).Scan(&noteID)
	if err != nil {
		return NoteDetail{}, err
	}

	return r.GetNote(ctx, userID, noteID)
}

func (r *PostgresNoteRepository) GetNote(ctx context.Context, userID, noteID string) (NoteDetail, error) {
	var note NoteDetail
	err := r.pool.QueryRow(
		ctx,
		`SELECT
			CAST(id AS TEXT) AS note_id,
			title,
			COALESCE(content, '') AS content,
			CAST(user_id AS TEXT) AS user_id,
			EXTRACT(EPOCH FROM created_at)::bigint AS created_at,
			EXTRACT(EPOCH FROM updated_at)::bigint AS updated_at
		FROM notes
		WHERE id = $1::int AND user_id = $2::int`,
		noteID,
		userID,
	).Scan(
		&note.NoteID,
		&note.Title,
		&note.Content,
		&note.UserID,
		&note.CreatedAt,
		&note.UpdatedAt,
	)
	if errors.Is(err, pgx.ErrNoRows) {
		return NoteDetail{}, ErrNotFound
	}
	if err != nil {
		return NoteDetail{}, err
	}

	tags, err := r.tagsForNote(ctx, noteID)
	if err != nil {
		return NoteDetail{}, err
	}
	note.Tags = tags
	return note, nil
}

func (r *PostgresNoteRepository) ListNotes(ctx context.Context, userID string) ([]NoteSummary, error) {
	rows, err := r.pool.Query(
		ctx,
		`SELECT
			CAST(id AS TEXT) AS note_id,
			title,
			LEFT(COALESCE(content, ''), 120) AS content_preview,
			CAST(user_id AS TEXT) AS user_id,
			EXTRACT(EPOCH FROM created_at)::bigint AS created_at,
			EXTRACT(EPOCH FROM updated_at)::bigint AS updated_at
		FROM notes
		WHERE user_id = $1::int
		ORDER BY updated_at DESC`,
		userID,
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var notes []NoteSummary
	for rows.Next() {
		var note NoteSummary
		if err := rows.Scan(
			&note.NoteID,
			&note.Title,
			&note.ContentPreview,
			&note.UserID,
			&note.CreatedAt,
			&note.UpdatedAt,
		); err != nil {
			return nil, err
		}
		notes = append(notes, note)
	}
	return notes, rows.Err()
}

func (r *PostgresNoteRepository) SearchNotes(ctx context.Context, userID, query string) ([]NoteSummary, error) {
	rows, err := r.pool.Query(
		ctx,
		`SELECT
			CAST(id AS TEXT) AS note_id,
			title,
			LEFT(COALESCE(content, ''), 120) AS content_preview,
			CAST(user_id AS TEXT) AS user_id,
			EXTRACT(EPOCH FROM created_at)::bigint AS created_at,
			EXTRACT(EPOCH FROM updated_at)::bigint AS updated_at
		FROM notes
		WHERE user_id = $1::int
		  AND (title ILIKE '%' || $2 || '%' OR COALESCE(content, '') ILIKE '%' || $2 || '%')
		ORDER BY updated_at DESC`,
		userID,
		query,
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var notes []NoteSummary
	for rows.Next() {
		var note NoteSummary
		if err := rows.Scan(
			&note.NoteID,
			&note.Title,
			&note.ContentPreview,
			&note.UserID,
			&note.CreatedAt,
			&note.UpdatedAt,
		); err != nil {
			return nil, err
		}
		notes = append(notes, note)
	}
	return notes, rows.Err()
}

func (r *PostgresNoteRepository) CreateTag(ctx context.Context, userID, name string) (Tag, error) {
	var tag Tag
	err := r.pool.QueryRow(
		ctx,
		`INSERT INTO tags (name, user_id)
		 VALUES ($1, $2::int)
		 RETURNING
			CAST(id AS TEXT) AS tag_id,
			name,
			CAST(user_id AS TEXT) AS user_id,
			EXTRACT(EPOCH FROM created_at)::bigint AS created_at`,
		name,
		userID,
	).Scan(&tag.TagID, &tag.Name, &tag.UserID, &tag.CreatedAt)
	if err != nil {
		var pgErr *pgconn.PgError
		if errors.As(err, &pgErr) && pgErr.Code == "23505" {
			return Tag{}, ErrAlreadyExists
		}
		return Tag{}, err
	}
	return tag, nil
}

func (r *PostgresNoteRepository) ListTags(ctx context.Context, userID string) ([]Tag, error) {
	rows, err := r.pool.Query(
		ctx,
		`SELECT
			CAST(id AS TEXT) AS tag_id,
			name,
			CAST(user_id AS TEXT) AS user_id,
			EXTRACT(EPOCH FROM created_at)::bigint AS created_at
		FROM tags
		WHERE user_id = $1::int
		ORDER BY name ASC`,
		userID,
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var tags []Tag
	for rows.Next() {
		var tag Tag
		if err := rows.Scan(&tag.TagID, &tag.Name, &tag.UserID, &tag.CreatedAt); err != nil {
			return nil, err
		}
		tags = append(tags, tag)
	}
	return tags, rows.Err()
}

func (r *PostgresNoteRepository) AttachTagToNote(ctx context.Context, userID, noteID, tagID string) (NoteDetail, error) {
	if _, err := r.GetNote(ctx, userID, noteID); err != nil {
		return NoteDetail{}, err
	}

	var exists bool
	err := r.pool.QueryRow(
		ctx,
		"SELECT EXISTS(SELECT 1 FROM tags WHERE id = $1::int AND user_id = $2::int)",
		tagID,
		userID,
	).Scan(&exists)
	if err != nil {
		return NoteDetail{}, err
	}
	if !exists {
		return NoteDetail{}, ErrNotFound
	}

	_, err = r.pool.Exec(
		ctx,
		`INSERT INTO note_tags (note_id, tag_id)
		 SELECT $1::int, $2::int
		 WHERE NOT EXISTS (
		   SELECT 1 FROM note_tags WHERE note_id = $1::int AND tag_id = $2::int
		 )`,
		noteID,
		tagID,
	)
	if err != nil {
		return NoteDetail{}, err
	}

	return r.GetNote(ctx, userID, noteID)
}

func (r *PostgresNoteRepository) tagsForNote(ctx context.Context, noteID string) ([]Tag, error) {
	rows, err := r.pool.Query(
		ctx,
		`SELECT
			CAST(t.id AS TEXT) AS tag_id,
			t.name,
			CAST(t.user_id AS TEXT) AS user_id,
			EXTRACT(EPOCH FROM t.created_at)::bigint AS created_at
		FROM tags t
		JOIN note_tags nt ON nt.tag_id = t.id
		WHERE nt.note_id = $1::int
		ORDER BY t.name ASC`,
		noteID,
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var tags []Tag
	for rows.Next() {
		var tag Tag
		if err := rows.Scan(&tag.TagID, &tag.Name, &tag.UserID, &tag.CreatedAt); err != nil {
			return nil, err
		}
		tags = append(tags, tag)
	}
	return tags, rows.Err()
}

func Preview(content string) string {
	content = strings.TrimSpace(content)
	if len(content) <= 120 {
		return content
	}
	return content[:120]
}
