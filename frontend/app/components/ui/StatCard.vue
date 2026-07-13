<template>
  <div class="stat-card group relative">
    <!-- Sophisticated Background Glow -->
    <div
      class="absolute top-0 right-0 h-32 w-32 rounded-full blur-[60px] opacity-10 pointer-events-none transition-opacity duration-700 group-hover:opacity-20"
      :style="`background: radial-gradient(circle, ${glowColors[stat?.color ?? 'brand']}, transparent); transform: translate(30%, -30%)`"
    />

    <div v-if="loading" class="space-y-4">
      <div class="skeleton h-10 w-10 rounded-2xl" />
      <div class="space-y-2">
        <div class="skeleton h-8 w-32 rounded-lg" />
        <div class="skeleton h-3 w-20 rounded" />
      </div>
    </div>

    <template v-else>
      <div
        class="h-12 w-12 rounded-2xl flex items-center justify-center mb-6 transition-transform duration-500 group-hover:scale-110 group-hover:rotate-3 shadow-lg"
        :class="iconBg[stat?.color ?? 'brand']"
      >
        <Icon :name="stat?.icon ?? 'heroicons:chart-bar'" class="h-6 w-6" :class="iconColor[stat?.color ?? 'brand']" />
      </div>

      <div class="space-y-1">
        <p class="text-4xl font-bold text-white font-display tracking-tight group-hover:text-brand-400 transition-colors">
          {{ stat?.value ?? '—' }}
        </p>
        <p class="text-[11px] font-black uppercase tracking-[0.2em] text-white/50">
          {{ stat?.label }}
        </p>
      </div>

      <div v-if="stat?.change !== undefined && stat.change !== 0" class="mt-6 flex items-center gap-2">
        <div class="flex items-center gap-1 px-2 py-1 rounded-full bg-white/[0.06] border border-white/[0.10] shadow-sm">
          <Icon
            :name="stat.change > 0 ? 'heroicons:arrow-trending-up' : 'heroicons:arrow-trending-down'"
            class="h-3.5 w-3.5"
            :class="stat.change > 0 ? 'text-emerald-400' : 'text-rose-400'"
          />
          <span class="text-[10px] font-bold" :class="stat.change > 0 ? 'text-emerald-400' : 'text-rose-400'">
            {{ stat.isMoney ? `R$ ${(Math.abs(stat.change) / 100).toFixed(0)}` : Math.abs(stat.change) }}
            {{ stat.changeLabel }}
          </span>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  stat?: {
    label: string
    value: string | number
    change?: number
    changeLabel?: string
    icon: string
    color: string
    isMoney?: boolean
  }
  loading?: boolean
}>()

const glowColors: Record<string, string> = {
  brand: '#7c3aed',
  emerald: '#10b981',
  blue: '#3b82f6',
  purple: '#a855f7',
  amber: '#f59e0b',
}

const iconBg: Record<string, string> = {
  brand: 'bg-brand-600/20',
  emerald: 'bg-emerald-500/15',
  blue: 'bg-blue-500/15',
  purple: 'bg-purple-500/15',
  amber: 'bg-amber-500/15',
}

const iconColor: Record<string, string> = {
  brand: 'text-brand-300',
  emerald: 'text-emerald-400',
  blue: 'text-blue-400',
  purple: 'text-purple-400',
  amber: 'text-amber-400',
}
</script>
