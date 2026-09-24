from pydantic import BaseModel


class OrderRequest(BaseModel):
    item: str
    quantity: int


class OrderRecord(BaseModel):
    order_id: int
    item: str
