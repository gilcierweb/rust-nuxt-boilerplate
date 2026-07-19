<script setup lang="ts">
definePageMeta({ layout: 'portal' })
const localePath = useLocalePath()
await navigateTo(localePath('/portal/dashboard'), { redirectCode: 302 })
</script>
