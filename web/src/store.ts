import { writable, Writable } from "svelte/store";

import { Status, Todo, TodoId } from "./model";
import Api from "./api";

export function createStore() {
  const todos: Writable<Todo[]> = writable([]);
  const editingTodo: Writable<Todo | null> = writable(null);
  const filter: Writable<null | Status> = writable(null);

  const reloadTodos = async () => {
    todos.set(await Api.find());
  };

  const createNew = async (text: string) => {
    await Api.create({
      text: text || "",
      status: Status.NotDone,
    });
  };

  const updateStatus = async (id: TodoId, status: Status) => {
    await Api.updateStatus(id.$oid, status);
  };

  const updateText = async (id: TodoId, text: string) => {
    await Api.updateText(id.$oid, text || "");
  };

  const removeTodo = async (id: TodoId) => {
    await Api.remove(id.$oid);
  };

  return {
    todos,
    editingTodo,
    createNew,
    reloadTodos,
    filter,
    removeTodo,
    updateStatus,
    updateText,
  };
}
