package main

import (
	"io"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
)

func main() {
	router := gin.Default()

	router.GET("/api/health", func(ctx *gin.Context) {
		ctx.JSON(http.StatusOK, gin.H{
			"ok":            true,
			"service":       "gin",
			"host":          ctx.Request.Host,
			"forwarded_for": ctx.GetHeader("X-Forwarded-For"),
			"time":          time.Now().UTC().Format(time.RFC3339),
		})
	})

	router.POST("/api/echo", func(ctx *gin.Context) {
		var body map[string]any
		_ = ctx.ShouldBindJSON(&body)
		ctx.JSON(http.StatusOK, gin.H{"received": body})
	})

	router.GET("/api/stream", func(ctx *gin.Context) {
		ctx.Stream(func(w io.Writer) bool {
			_, _ = w.Write([]byte("gate\n"))
			_, _ = w.Write([]byte("gin\n"))
			return false
		})
	})

	if err := router.Run("127.0.0.1:8080"); err != nil {
		panic(err)
	}
}
