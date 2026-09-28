<script lang="ts">
	import { browser } from '$app/environment';
	import { goto } from '$app/navigation';

	let locale = $state('eng');

	if (browser) {
		const language = navigator.language.toLowerCase();

		locale =
			language.startsWith('cs') ? 'cze' :
			language.startsWith('uk') ? 'ukr' :
			'eng';
	}
</script>

<p>Redirecting to {locale}...</p>
