<script lang="ts">
	import {onMount} from "svelte";
	import { createStore } from "./store";
	import { Status, Todo ,TodoId } from "./model";

	const ENTER_KEY = 13;
	const ESCAPE_KEY = 27;

	const { todos, reloadTodos, editingTodo, filter, updateStatus, updateText ,createNew ,removeTodo} = createStore();

	onMount(async () =>{
			await reloadTodos()
	});

	async function create(event : any){
		if (event.which === ENTER_KEY){
				await createNew(event.target.value)
				event.target.value = "";
				await reloadTodos()
		}
	}

	async function remove(id? : TodoId){
		await removeTodo(id as TodoId);
		await reloadTodos()
	}

	async function handleEdit(event:any){
		if (event.which === ENTER_KEY){
				if ($editingTodo){
					await submitEdit($editingTodo._id as TodoId, event.target.value);
					await reloadTodos()
				}else{
					console.log("editing is not se")
				}
		}else if (event.which === ESCAPE_KEY){
			if ($editingTodo){
				 event.target.value = $editingTodo.text
			   editingTodo.set(null);
			}
		}
	}

	async function submitEdit(id :TodoId, text:string){
		await updateText(id,text);
		editingTodo.set(null);
		await reloadTodos();
	}

	async function onBlurEdit(){
		editingTodo.set(null);
	}

	async function setEditing(todo : Todo){
			editingTodo.set(todo)
	}

	async function clearCompleted(){
		const targets = $todos.filter(item => item.status == Status.Done)
		await Promise.all(targets.map(todo => removeTodo(todo._id as TodoId)));
		await reloadTodos();
	}

	function setFilter(newFilter:Status|null){
		filter.set(newFilter)
	}
	async function toggleCheck(todo:Todo){
		const newStatus = todo.status == Status.Done ? Status.NotDone : Status.Done;
		await updateStatus(todo._id as TodoId, newStatus );
		await reloadTodos();
	}

	$: filteredTodos = $todos.filter(item =>	$filter ?  item.status == $filter: item );
	$: numActive = $todos.filter(item => item.status == Status.NotDone).length;
	$: numCompleted = $todos.filter(item => item.status == Status.Done).length;

</script>


<header class="header">
	<h1>todos</h1>
	<input
		class="new-todo"
		on:keydown={create}
		placeholder="What needs to be done?"
	>
</header>

{#if $todos.length > 0}
	<section class="main">
		<ul class="todo-list">
			{#each filteredTodos as todo }
				<li class="{todo.status == Status.Done ? 'completed' : ''} { $editingTodo ? ($editingTodo._id === todo._id ? 'editing' : '' ) :''}">
					<div class="view">
						<input name ="todoCheck" class="toggle" type="checkbox" checked={todo.status == Status.Done} on:click="{ () => toggleCheck(todo)}">
						<label for="todoCheck" on:dblclick="{() => setEditing(todo)}">{todo.text}</label>
						<button on:click="{async () => remove(todo._id)}" class="destroy"></button>
					</div>

					{#if $editingTodo !== null }
						{#if $editingTodo._id === todo._id}
							<input
								value='{todo.text}'
								id="edit"
								class="edit"
								on:keydown={handleEdit}
								on:blur={onBlurEdit}
							>
						{/if}
					{/if}
				</li>
			{/each}
		</ul>

		<footer class="footer">
			<span class="todo-count">
				<strong>{numActive}</strong> {numActive === 1 ? 'item' : 'items'} left
			</span>

			<ul class="filters">
				<li><a class="{$filter === null ? 'selected' : ''}" href="/#" on:click="{()=> setFilter(null)}">All</a></li>
				<li><a class="{$filter === Status.NotDone ? 'selected' : ''}" href="/#" on:click="{()=> setFilter(Status.NotDone)}">Active</a></li>
				<li><a class="{$filter === Status.Done ? 'selected' : ''}" href="/#" on:click="{()=> setFilter(Status.Done)}">Completed</a></li>
			</ul>

			{#if numCompleted}
				<button class="clear-completed" on:click={clearCompleted}>
					Clear completed
				</button>
			{/if}
		</footer>
	</section>
{/if}
