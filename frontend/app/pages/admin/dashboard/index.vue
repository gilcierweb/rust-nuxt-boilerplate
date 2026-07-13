<template>
  <section class="space-y-6">
    <AdminBreadcrumb :items="breadcrumbItems" />

    <div class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5">
      <div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <div class="mb-3 inline-flex items-center gap-2 rounded-field bg-primary/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.22em] text-primary">
            <span class="icon-[tabler--layout-dashboard] size-4"></span>
            <span>{{ $t('admin.section') }}</span>
          </div>
          <h1 class="text-3xl font-semibold tracking-tight text-base-content">{{ $t('admin.title') }}</h1>
          <p class="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">
            Monitoramento rápido de clientes, débitos, pagamentos, documentos e solicitações fiscais.
          </p>
        </div>

        <div class="flex flex-wrap gap-2">
          <button type="button" class="btn btn-soft" :disabled="pendingAny" @click="refreshDashboard">
            <span class="icon-[tabler--refresh] size-4.5"></span>
            Atualizar
          </button>
          <NuxtLink to="/admin/payment-transactions/new" class="btn btn-primary">
            <span class="icon-[tabler--plus] size-4.5"></span>
            Nova Transação
          </NuxtLink>
        </div>
      </div>
    </div>

    <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
      <div class="rounded-box border border-base-content/10 bg-base-100 p-4 shadow-md shadow-base-content/5">
        <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('admin.dashboard.activeCustomers') }}</p>
        <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.activeCustomers }}</p>
      </div>
      <div class="rounded-box border border-base-content/10 bg-base-100 p-4 shadow-md shadow-base-content/5">
        <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('admin.dashboard.openDebts') }}</p>
        <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.openDebts }}</p>
      </div>
      <div class="rounded-box border border-base-content/10 bg-base-100 p-4 shadow-md shadow-base-content/5">
        <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('admin.dashboard.pendingPayments') }}</p>
        <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.pendingTransactions }}</p>
      </div>
      <div class="rounded-box border border-base-content/10 bg-base-100 p-4 shadow-md shadow-base-content/5">
        <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('admin.dashboard.pendingInvoices') }}</p>
        <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.pendingInvoiceRequests }}</p>
      </div>
    </div>

    <div class="grid gap-6 xl:grid-cols-3">
      <div class="rounded-box border border-base-content/10 bg-base-100 p-5 shadow-md shadow-base-content/5 xl:col-span-2">
        <div class="mb-4 flex items-center justify-between gap-3">
          <h2 class="text-lg font-semibold text-base-content">{{ $t('admin.dashboard.financialSummary') }}</h2>
        </div>
        <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          <div class="rounded-box bg-base-200/70 p-4">
            <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('admin.dashboard.totalDebts') }}</p>
            <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.totalDebts }}</p>
          </div>
          <div class="rounded-box bg-base-200/70 p-4">
            <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('admin.dashboard.completedPayments') }}</p>
            <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.settledTransactions }}</p>
          </div>
          <div class="rounded-box bg-base-200/70 p-4">
            <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">Vencendo em 7 dias</p>
            <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.dueSoonDebts }}</p>
          </div>
        </div>
      </div>

      <div class="rounded-box border border-base-content/10 bg-base-100 p-5 shadow-md shadow-base-content/5">
        <h2 class="text-lg font-semibold text-base-content">{{ $t('admin.sidebar.documents') }}</h2>
        <div class="mt-4 space-y-4">
          <div class="rounded-box bg-base-200/70 p-4">
            <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('admin.common.total') }}</p>
            <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.totalDocuments }}</p>
          </div>
          <div class="rounded-box bg-base-200/70 p-4">
            <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('admin.dashboard.visibleToCustomer') }}</p>
            <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.visibleDocuments }}</p>
          </div>
          <NuxtLink to="/admin/documents/new" class="btn btn-soft w-full">
            Novo Documento
          </NuxtLink>
        </div>
      </div>
    </div>

    <div class="rounded-box border border-base-content/10 bg-base-100 p-5 shadow-md shadow-base-content/5">
      <div class="mb-4 flex items-center justify-between">
        <h2 class="text-lg font-semibold text-base-content">{{ $t('admin.dashboard.recentActivity') }}</h2>
        <NuxtLink to="/admin/invoice-requests/new" class="btn btn-soft btn-sm">
          Nova Solicitação de Nota
        </NuxtLink>
      </div>

      <ul v-if="recentActivity.length > 0" class="space-y-3">
        <li
          v-for="item in recentActivity"
          :key="item.key"
          class="rounded-box border border-base-content/10 bg-base-200/40 px-4 py-3"
        >
          <div class="flex flex-col gap-1 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <p class="font-medium text-base-content">{{ item.title }}</p>
              <p class="text-sm text-base-content/65">{{ item.description }}</p>
            </div>
            <span class="text-sm text-base-content/60">{{ formatDateTime(item.date) }}</span>
          </div>
        </li>
      </ul>
      <p v-else class="text-sm text-base-content/60">{{ $t('admin.dashboard.noRecentActivity') }}</p>
    </div>
  </section>
</template>

<script setup lang="ts">
import { formatDateTime } from '~/utils/admin-ui'
import { normalizeResourceResponse } from '~/utils/admin-resources'

const { t } = useI18n()

definePageMeta({
  layout: 'admin',
})

const breadcrumbItems = computed(() => [{ label: t('admin.common.dashboard') }])

const customersFetch = await useApiFetch<any>('/admin/customers', { key: 'admin-dashboard-customers', server: true, default: () => [] })
const debtsFetch = await useApiFetch<any>('/admin/debts', { key: 'admin-dashboard-debts', server: true, default: () => [] })
const paymentTransactionsFetch = await useApiFetch<any>('/admin/payment-transactions', { key: 'admin-dashboard-payment-transactions', server: true, default: () => [] })
const documentsFetch = await useApiFetch<any>('/admin/documents', { key: 'admin-dashboard-documents', server: true, default: () => [] })
const invoiceRequestsFetch = await useApiFetch<any>('/admin/invoice-requests', { key: 'admin-dashboard-invoice-requests', server: true, default: () => [] })

const customers = computed<any[]>(() => normalizeResourceResponse(customersFetch.data.value))
const debts = computed<any[]>(() => normalizeResourceResponse(debtsFetch.data.value))
const paymentTransactions = computed<any[]>(() => normalizeResourceResponse(paymentTransactionsFetch.data.value))
const documents = computed<any[]>(() => normalizeResourceResponse(documentsFetch.data.value))
const invoiceRequests = computed<any[]>(() => normalizeResourceResponse(invoiceRequestsFetch.data.value))

const pendingAny = computed(() =>
  customersFetch.pending.value
  || debtsFetch.pending.value
  || paymentTransactionsFetch.pending.value
  || documentsFetch.pending.value
  || invoiceRequestsFetch.pending.value,
)

function parseDate(value?: string): number {
  if (!value) return 0
  const parsed = new Date(value).getTime()
  return Number.isNaN(parsed) ? 0 : parsed
}

const stats = computed(() => {
  const activeCustomers = customers.value.filter((item) => Number(item.status) === 1).length
  const totalDebts = debts.value.length
  const openDebts = debts.value.filter((item) => Number(item.status) !== 3).length
  const pendingTransactions = paymentTransactions.value.filter((item) => [1, 2].includes(Number(item.status))).length
  const settledTransactions = paymentTransactions.value.filter((item) => Number(item.status) === 3).length
  const pendingInvoiceRequests = invoiceRequests.value.filter((item) => Number(item.status) === 1).length
  const totalDocuments = documents.value.length
  const visibleDocuments = documents.value.filter((item) => item.is_visible_to_customer === true).length

  const now = Date.now()
  const dueSoonLimit = now + (7 * 24 * 60 * 60 * 1000)
  const dueSoonDebts = debts.value.filter((item) => {
    const dueAt = parseDate(item.due_date)
    return dueAt > now && dueAt <= dueSoonLimit && Number(item.status) !== 3
  }).length

  return {
    activeCustomers,
    totalDebts,
    openDebts,
    pendingTransactions,
    settledTransactions,
    pendingInvoiceRequests,
    totalDocuments,
    visibleDocuments,
    dueSoonDebts,
  }
})

const recentActivity = computed(() => {
  const transactions = paymentTransactions.value.map((item) => ({
    key: `payment-${item.id}`,
    date: item.updated_at || item.created_at,
    title: 'Transação de pagamento',
    description: `Status ${item.status} - ${item.provider_reference || item.id}`,
  }))

  const invoices = invoiceRequests.value.map((item) => ({
    key: `invoice-request-${item.id}`,
    date: item.updated_at || item.created_at,
    title: 'Solicitação de nota',
    description: item.service_description || item.id,
  }))

  const docs = documents.value.map((item) => ({
    key: `document-${item.id}`,
    date: item.updated_at || item.created_at,
    title: 'Documento',
    description: item.title || item.id,
  }))

  return [...transactions, ...invoices, ...docs]
    .sort((left, right) => parseDate(right.date) - parseDate(left.date))
    .slice(0, 8)
})

async function refreshDashboard() {
  await Promise.all([
    customersFetch.refresh(),
    debtsFetch.refresh(),
    paymentTransactionsFetch.refresh(),
    documentsFetch.refresh(),
    invoiceRequestsFetch.refresh(),
  ])
}
</script>
