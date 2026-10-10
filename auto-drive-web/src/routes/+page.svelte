<script lang="ts">
	import { browser } from '$app/environment';

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
