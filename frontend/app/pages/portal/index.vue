<script setup lang="ts">
definePageMeta({ layout: 'portal' })
const localePath = useLocalePath()
const route = useRoute()

// This page is a hop to the dashboard, and a redirect built from a path
// drops the query: a flash notice that already travelled one hop (the
// middleware that refused an admin screen) used to die here. The notice is
// forwarded, alone, so the dashboard still receives it.
await navigateTo(
  {
    path: localePath('/portal/dashboard'),
    query: noticeQuery(route.query),
  },
  { redirectCode: 302 },
)
</script>
