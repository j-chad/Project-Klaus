<script module>
    import { defineMeta } from '@storybook/addon-svelte-csf';
    import Snow from './Snow.svelte';

    const { Story } = defineMeta({
        title: 'Snow',
        component: Snow,
        tags: ['autodocs'],
        globals: {
            backgrounds: {
                value: 'dark',
            },
        }
    });
</script>

<Story name="Primary"/>