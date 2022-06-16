export enum Status {
  Done = "DONE",
  NotDone = "NOT_DONE",
}

export interface TodoId {
  $oid: string;
}

export interface Todo {
  _id?: TodoId;
  text: string;
  status: Status;
}
