<script setup lang="ts">
definePageMeta({
  layout: 'admin',
})

await navigateTo('/admin/dashboard', { redirectCode: 302 })
</script>
