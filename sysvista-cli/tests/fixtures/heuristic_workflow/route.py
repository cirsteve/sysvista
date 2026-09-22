from models import User

@app.get("/users", response_model=User)
def list_users():
    return []
