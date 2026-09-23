from models import OrderRequest, OrderRecord


def to_order_record(request: OrderRequest) -> OrderRecord:
    return OrderRecord(order_id=1, item=request.item)
