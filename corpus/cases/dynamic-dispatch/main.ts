export function invoke(receiver: any): unknown {
  return receiver.execute();
}
