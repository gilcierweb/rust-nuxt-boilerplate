<template>
  <section class="space-y-6">
    <div class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5">
      <h1 class="text-2xl font-semibold text-base-content">{{ $t('portal.dashboard.title') }}</h1>
      <p class="mt-2 text-sm text-base-content/65">
        Acompanhe sua posição financeira, documentos e solicitações fiscais.
      </p>
    </div>

    <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
      <div class="rounded-box border border-base-content/10 bg-base-100 p-4">
        <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('portal.dashboard.openDebts') }}</p>
        <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.openDebts }}</p>
      </div>
      <div class="rounded-box border border-base-content/10 bg-base-100 p-4">
        <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('portal.dashboard.pendingPayments') }}</p>
        <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.pendingPayments }}</p>
      </div>
      <div class="rounded-box border border-base-content/10 bg-base-100 p-4">
        <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('portal.dashboard.availableDocuments') }}</p>
        <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.availableDocuments }}</p>
      </div>
      <div class="rounded-box border border-base-content/10 bg-base-100 p-4">
        <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('portal.dashboard.openInvoiceRequests') }}</p>
        <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.openInvoiceRequests }}</p>
      </div>
    </div>

    <div class="grid gap-6 xl:grid-cols-2">
      <div class="rounded-box border border-base-content/10 bg-base-100 p-5">
        <div class="mb-4 flex items-center justify-between">
          <h2 class="text-lg font-semibold text-base-content">{{ $t('portal.dashboard.upcomingDueDates') }}</h2>
          <NuxtLink to="/portal/debts" class="btn btn-soft btn-sm">{{ $t('portal.dashboard.viewDebts') }}</NuxtLink>
        </div>
        <ul v-if="upcomingDebts.length" class="space-y-3">
          <li v-for="debt in upcomingDebts" :key="debt.id" class="rounded-box border border-base-content/10 bg-base-200/40 px-4 py-3">
            <div class="flex items-start justify-between gap-4">
              <div>
                <p class="font-medium text-base-content">{{ debt.title || debt.external_reference || debt.id }}</p>
                <p class="text-sm text-base-content/60">Vencimento: {{ formatDate(debt.due_date) }}</p>
              </div>
              <p class="font-semibold text-base-content">{{ formatCurrency(debt.amount) }}</p>
            </div>
          </li>
        </ul>
        <p v-else class="text-sm text-base-content/60">{{ $t('portal.dashboard.noUpcomingDebts') }}</p>
      </div>

      <div class="rounded-box border border-base-content/10 bg-base-100 p-5">
        <div class="mb-4 flex items-center justify-between">
          <h2 class="text-lg font-semibold text-base-content">{{ $t('portal.dashboard.recentActivity') }}</h2>
          <NuxtLink to="/portal/payments" class="btn btn-soft btn-sm">{{ $t('portal.dashboard.viewPayments') }}</NuxtLink>
        </div>
        <ul v-if="recentActivity.length" class="space-y-3">
          <li v-for="entry in recentActivity" :key="entry.key" class="rounded-box border border-base-content/10 bg-base-200/40 px-4 py-3">
            <p class="font-medium text-base-content">{{ entry.title }}</p>
            <p class="text-sm text-base-content/60">{{ entry.description }}</p>
            <p class="mt-1 text-xs text-base-content/50">{{ formatDateTime(entry.date) }}</p>
          </li>
        </ul>
        <p v-else class="text-sm text-base-content/60">{{ $t('portal.dashboard.noRecentActivity') }}</p>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { formatCurrency, formatDate, formatDateTime } from '~/utils/admin-ui'
import { normalizeResourceResponse } from '~/utils/admin-resources'

definePageMeta({ layout: 'portal' })

const debtsFetch = await useApiFetch<any>('/admin/debts', {
  key: 'portal-dashboard-debts',
  server: true,
  default: () => [],
})
const paymentsFetch = await useApiFetch<any>('/admin/payment-transactions', {
  key: 'portal-dashboard-payments',
  server: true,
  default: () => [],
})
const documentsFetch = await useApiFetch<any>('/admin/documents', {
  key: 'portal-dashboard-documents',
  server: true,
  default: () => [],
})
const invoiceRequestsFetch = await useApiFetch<any>('/admin/invoice-requests', {
  key: 'portal-dashboard-invoice-requests',
  server: true,
  default: () => [],
})

const debts = computed<any[]>(() => normalizeResourceResponse(debtsFetch.data.value))
const payments = computed<any[]>(() => normalizeResourceResponse(paymentsFetch.data.value))
const documents = computed<any[]>(() => normalizeResourceResponse(documentsFetch.data.value))
const invoiceRequests = computed<any[]>(() => normalizeResourceResponse(invoiceRequestsFetch.data.value))

function parseDate(value?: string): number {
  if (!value) return 0
  const parsed = new Date(value).getTime()
  return Number.isNaN(parsed) ? 0 : parsed
}

const stats = computed(() => ({
  openDebts: debts.value.filter((item) => Number(item.status) !== 3).length,
  pendingPayments: payments.value.filter((item) => [1, 2].includes(Number(item.status))).length,
  availableDocuments: documents.value.filter((item) => item.is_visible_to_customer === true).length,
  openInvoiceRequests: invoiceRequests.value.filter((item) => Number(item.status) === 1).length,
}))

const upcomingDebts = computed(() =>
  debts.value
    .filter((item) => Number(item.status) !== 3)
    .slice()
    .sort((a, b) => parseDate(a.due_date) - parseDate(b.due_date))
    .slice(0, 5),
)

const recentActivity = computed(() => {
  const paymentEntries = payments.value.map((item) => ({
    key: `payment-${item.id}`,
    date: item.updated_at || item.created_at,
    title: 'Pagamento',
    description: `Status ${item.status}${item.provider_reference ? ` · ${item.provider_reference}` : ''}`,
  }))

  const invoiceEntries = invoiceRequests.value.map((item) => ({
    key: `invoice-${item.id}`,
    date: item.updated_at || item.created_at,
    title: 'Solicitação de nota',
    description: item.service_description || item.id,
  }))

  return [...paymentEntries, ...invoiceEntries]
    .sort((a, b) => parseDate(b.date) - parseDate(a.date))
    .slice(0, 6)
})
</script>
