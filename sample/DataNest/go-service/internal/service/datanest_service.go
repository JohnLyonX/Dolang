package service

import (
	"context"
	"errors"

	pb "datanest/go-service/internal/pb"
	"datanest/go-service/internal/repository"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

type DataNestServer struct {
	pb.UnimplementedDataNestServiceServer
	repo repository.NoteRepository
}

func NewDataNestServer(repo repository.NoteRepository) *DataNestServer {
	return &DataNestServer{repo: repo}
}

func userFromMetadata(ctx context.Context) (string, string, error) {
	md, ok := metadata.FromIncomingContext(ctx)
	if !ok {
		return "", "", status.Error(codes.Unauthenticated, "missing metadata")
	}

	userIDs := md.Get("x-user-id")
	if len(userIDs) == 0 || userIDs[0] == "" {
		return "", "", status.Error(codes.Unauthenticated, "missing x-user-id")
	}

	usernames := md.Get("x-username")
	username := ""
	if len(usernames) > 0 {
		username = usernames[0]
	}

	return userIDs[0], username, nil
}

func (s *DataNestServer) CreateNote(ctx context.Context, req *pb.CreateNoteRequest) (*pb.NoteDetail, error) {
	userID, _, err := userFromMetadata(ctx)
	if err != nil {
		return nil, err
	}
	if req.GetTitle() == "" {
		return nil, status.Error(codes.InvalidArgument, "title is required")
	}

	note, err := s.repo.CreateNote(ctx, userID, req.GetTitle(), req.GetContent())
	if err != nil {
		return nil, mapError(err)
	}
	return noteDetailToPB(note), nil
}

func (s *DataNestServer) GetNote(ctx context.Context, req *pb.GetNoteRequest) (*pb.NoteDetail, error) {
	userID, _, err := userFromMetadata(ctx)
	if err != nil {
		return nil, err
	}
	if req.GetNoteId() == "" {
		return nil, status.Error(codes.InvalidArgument, "note_id is required")
	}

	note, err := s.repo.GetNote(ctx, userID, req.GetNoteId())
	if err != nil {
		return nil, mapError(err)
	}
	return noteDetailToPB(note), nil
}

func (s *DataNestServer) ListNotes(ctx context.Context, _ *pb.ListNotesRequest) (*pb.ListNotesResponse, error) {
	userID, _, err := userFromMetadata(ctx)
	if err != nil {
		return nil, err
	}

	notes, err := s.repo.ListNotes(ctx, userID)
	if err != nil {
		return nil, mapError(err)
	}
	return &pb.ListNotesResponse{Notes: noteSummariesToPB(notes)}, nil
}

func (s *DataNestServer) SearchNotes(ctx context.Context, req *pb.SearchNotesRequest) (*pb.ListNotesResponse, error) {
	userID, _, err := userFromMetadata(ctx)
	if err != nil {
		return nil, err
	}

	notes, err := s.repo.SearchNotes(ctx, userID, req.GetQuery())
	if err != nil {
		return nil, mapError(err)
	}
	return &pb.ListNotesResponse{Notes: noteSummariesToPB(notes)}, nil
}

func (s *DataNestServer) CreateTag(ctx context.Context, req *pb.CreateTagRequest) (*pb.Tag, error) {
	userID, _, err := userFromMetadata(ctx)
	if err != nil {
		return nil, err
	}
	if req.GetName() == "" {
		return nil, status.Error(codes.InvalidArgument, "name is required")
	}

	tag, err := s.repo.CreateTag(ctx, userID, req.GetName())
	if err != nil {
		return nil, mapError(err)
	}
	return tagToPB(tag), nil
}

func (s *DataNestServer) ListTags(ctx context.Context, _ *pb.ListTagsRequest) (*pb.ListTagsResponse, error) {
	userID, _, err := userFromMetadata(ctx)
	if err != nil {
		return nil, err
	}

	tags, err := s.repo.ListTags(ctx, userID)
	if err != nil {
		return nil, mapError(err)
	}
	return &pb.ListTagsResponse{Tags: tagsToPB(tags)}, nil
}

func (s *DataNestServer) AttachTagToNote(ctx context.Context, req *pb.AttachTagToNoteRequest) (*pb.NoteDetail, error) {
	userID, _, err := userFromMetadata(ctx)
	if err != nil {
		return nil, err
	}
	if req.GetNoteId() == "" || req.GetTagId() == "" {
		return nil, status.Error(codes.InvalidArgument, "note_id and tag_id are required")
	}

	note, err := s.repo.AttachTagToNote(ctx, userID, req.GetNoteId(), req.GetTagId())
	if err != nil {
		return nil, mapError(err)
	}
	return noteDetailToPB(note), nil
}

func mapError(err error) error {
	switch {
	case errors.Is(err, repository.ErrNotFound):
		return status.Error(codes.NotFound, "resource not found")
	case errors.Is(err, repository.ErrAlreadyExists):
		return status.Error(codes.AlreadyExists, "resource already exists")
	default:
		return status.Error(codes.Internal, err.Error())
	}
}

func tagToPB(tag repository.Tag) *pb.Tag {
	return &pb.Tag{
		TagId:     tag.TagID,
		Name:      tag.Name,
		UserId:    tag.UserID,
		CreatedAt: tag.CreatedAt,
	}
}

func tagsToPB(tags []repository.Tag) []*pb.Tag {
	out := make([]*pb.Tag, 0, len(tags))
	for _, tag := range tags {
		out = append(out, tagToPB(tag))
	}
	return out
}

func noteDetailToPB(note repository.NoteDetail) *pb.NoteDetail {
	return &pb.NoteDetail{
		NoteId:    note.NoteID,
		Title:     note.Title,
		Content:   note.Content,
		UserId:    note.UserID,
		CreatedAt: note.CreatedAt,
		UpdatedAt: note.UpdatedAt,
		Tags:      tagsToPB(note.Tags),
	}
}

func noteSummariesToPB(notes []repository.NoteSummary) []*pb.NoteSummary {
	out := make([]*pb.NoteSummary, 0, len(notes))
	for _, note := range notes {
		out = append(out, &pb.NoteSummary{
			NoteId:         note.NoteID,
			Title:          note.Title,
			ContentPreview: note.ContentPreview,
			UserId:         note.UserID,
			CreatedAt:      note.CreatedAt,
			UpdatedAt:      note.UpdatedAt,
		})
	}
	return out
}
