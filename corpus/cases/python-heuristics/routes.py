from fastapi import APIRouter, BackgroundTasks

from models import OrderRequest, OrderRecord
import transforms
from services.notify import send_receipt

router = APIRouter()


@router.post("/orders", response_model=OrderRecord)
async def create_order(request: OrderRequest, background_tasks: BackgroundTasks) -> OrderRecord:
    record = transforms.to_order_record(request)
    background_tasks.add_task(send_receipt, record)
    return record


class OrderController:
    def list(self) -> list:
        return []
