from langchain.prompts import PromptTemplate

from models import OrderRecord


class OrderService:
    summary = PromptTemplate("Summarize the order for {item}")

    def summarize(self, record: OrderRecord) -> str:
        return self.summary.format(item=record.item)
