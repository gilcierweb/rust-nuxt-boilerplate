<template>
  <div
    class="relative flex min-h-screen items-center justify-center overflow-x-hidden bg-base-200/60 bg-cover bg-center bg-no-repeat py-10"
  >
    <div class="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_20%_20%,color-mix(in_oklab,var(--color-primary)_16%,transparent),transparent_40%),radial-gradient(circle_at_80%_80%,color-mix(in_oklab,var(--color-primary)_10%,transparent),transparent_36%)]" />

    <div class="relative z-10 flex w-full max-w-xl items-center justify-center px-4 sm:px-6 lg:px-8">
      <div class="pointer-events-none absolute -z-10 opacity-90">
        <svg width="612" height="697" viewBox="0 0 612 697" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
          <path
            d="M360.405 111.996C393.955 67.9448 456.863 59.4318 500.914 92.9818V92.9818C544.965 126.532 553.478 189.44 519.928 233.491L250.545 587.191C216.995 631.243 154.087 639.756 110.036 606.206V606.206C65.9845 572.656 57.4716 509.747 91.0216 465.696L360.405 111.996Z"
            fill="url(#paint0_linear_auth)"
            fill-opacity="0.08"
          />
          <path
            d="M519.53 233.188L250.147 586.888C216.765 630.72 154.17 639.19 110.339 605.808C66.5071 572.425 58.0367 509.831 91.4194 465.999L360.802 112.299C394.185 68.4674 456.78 59.9969 500.611 93.3796C544.443 126.762 552.913 189.357 519.53 233.188Z"
            stroke="var(--color-primary)"
            stroke-opacity="0.22"
          />
          <defs>
            <linearGradient id="paint0_linear_auth" x1="500.914" y1="92.9818" x2="110.036" y2="606.206" gradientUnits="userSpaceOnUse">
              <stop offset="0" stop-color="var(--color-primary)" />
              <stop offset="1" stop-color="var(--color-primary)" stop-opacity="0.2" />
            </linearGradient>
          </defs>
        </svg>
      </div>

      <div class="w-full space-y-6 rounded-xl bg-base-100 p-6 shadow-md shadow-base-300/20 lg:p-8">
        <slot />
      </div>
    </div>
  </div>
</template>
