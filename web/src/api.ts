import { Status, Todo } from "model";

const apiBaseUrl = process.env.API_BASE_URL;

export async function find(): Promise<Todo[]> {
  return await fetch(`${apiBaseUrl}api/todos`)
    .then(async resp => {
      const result = await resp.json();
      return result as Todo[];
    })
    .catch(error => {
      console.log(error);
      return [] as Todo[];
    });
}

export async function create(todo: Todo): Promise<Todo | null> {
  return await fetch(`${apiBaseUrl}api/todos`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(todo),
  })
    .then(async resp => {
      const result = await resp.json();
      return result as Todo;
    })
    .catch(error => {
      console.log(error);
      return null;
    });
}

export async function updateStatus(id: string, status: Status) {
  return await fetch(`${apiBaseUrl}api/todos/${id}/status`, {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ status }),
  }).catch(error => {
    console.log(error);
  });
}

export async function updateText(id: string, text: string) {
  return await fetch(`${apiBaseUrl}api/todos/${id}`, {
    method: "PUT",
    body: JSON.stringify({ text }),
    headers: {
      "Content-Type": "application/json",
    },
  }).catch(error => {
    console.log(error);
  });
}

export async function remove(id: string) {
  return await fetch(`${apiBaseUrl}api/todos/${id}`, {
    method: "DELETE",
    headers: {
      "Content-Type": "application/json",
    },
  }).catch(error => {
    console.log(error);
  });
}

export default {
  find,
  create,
  remove,
  updateStatus,
  updateText,
};
