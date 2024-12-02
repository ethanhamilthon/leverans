<script lang="ts">
	import PocketBase from 'pocketbase';
	import { onMount } from 'svelte';
	let messages: {
		content: string;
		username: string;
	}[] = $state([]);
	let text = $state('');
	let username = $state('');
	onMount(() => {
		const pb = new PocketBase(import.meta.env.VITE_POCKETBASE_URL);
		pb.collection('message')
			.getFullList()
			.then((records) => {
				console.log(records);
				messages = records.map((r) => {
					return {
						content: r.content,
						username: r.username
					};
				});
			});
		pb.collection('message').subscribe('*', (e) => {
			console.log(e.record);
			messages = [
				...messages,
				{
					content: e.record.content,
					username: e.record.username
				}
			];
		});
	});
</script>

<div class="flex h-screen flex-col items-center justify-center bg-gray-100">
	<input
		class="w-96 rounded-md border border-gray-300 p-2"
		type="text"
		bind:value={username}
		placeholder="username"
	/>
	<div class="flex min-h-96 w-96 flex-col justify-between rounded-md bg-white p-4">
		<div class="flex w-full flex-col gap-3">
			{#each messages as message}
				<ul class="flex flex-col gap-1 rounded-sm bg-gray-100 p-2">
					<li class="text-xs font-light text-gray-500">User: {message.username}</li>
					<li>{message.content}</li>
				</ul>
			{/each}
		</div>
		<div class="flex w-full">
			<input
				type="text"
				bind:value={text}
				class="w-full flex-1 rounded border border-gray-300 p-2"
			/>

			<button
				class="ml-2 rounded bg-blue-500 px-4 py-2 font-bold text-white hover:bg-blue-700"
				onclick={() => {
					const pb = new PocketBase(import.meta.env.VITE_POCKETBASE_URL);
					pb.collection('message').create({ content: text, username: username });
					text = '';
				}}>Send</button
			>
		</div>
	</div>
</div>
