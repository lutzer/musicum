from fastapi import FastAPI

app = FastAPI(
    title="Musicum API",
    description="API for organizing sound recordings and publishing collections",
    version="0.1.0",
)


@app.get("/health")
def health_check():
    return {"status": "healthy"}
