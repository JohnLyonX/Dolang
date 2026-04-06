package main

import (
	"context"
	"log"
	"net"
	"os"

	"datanest/go-service/internal/db"
	pb "datanest/go-service/internal/pb"
	"datanest/go-service/internal/repository"
	"datanest/go-service/internal/service"

	"google.golang.org/grpc"
)

func main() {
	addr := os.Getenv("DATANEST_GRPC_ADDR")
	if addr == "" {
		addr = ":50051"
	}

	databaseURL := os.Getenv("DATANEST_DATABASE_URL")
	if databaseURL == "" {
		log.Fatal("DATANEST_DATABASE_URL is required")
	}

	ctx := context.Background()
	pool, err := db.NewPool(ctx, databaseURL)
	if err != nil {
		log.Fatalf("connect postgres: %v", err)
	}
	defer pool.Close()

	repo := repository.NewPostgresNoteRepository(pool)
	server := grpc.NewServer()
	pb.RegisterDataNestServiceServer(server, service.NewDataNestServer(repo))

	lis, err := net.Listen("tcp", addr)
	if err != nil {
		log.Fatalf("listen %s: %v", addr, err)
	}

	log.Printf("DataNest gRPC listening on %s", addr)
	log.Fatal(server.Serve(lis))
}
