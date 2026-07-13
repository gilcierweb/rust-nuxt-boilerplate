<template>
  <!-- Section 14: Detailed Footer -->
  <footer class="bg-[#F7FAFC] dark:bg-slate-950 py-12 px-6 border-t border-slate-200 dark:border-slate-800">
    <div class="max-w-[1440px] mx-auto grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-8 mb-12">
      <div class="col-span-2">
        <div class="flex items-center gap-2 mb-4">
          <span class="material-symbols-outlined text-[#2F80ED] text-3xl" data-icon="local_laundry_service">local_laundry_service</span>
          <span class="text-xl font-bold text-slate-900 dark:text-white font-display-xl tracking-tight">iLaundry</span>
        </div>
        <p class="text-sm font-['Inter'] leading-relaxed text-slate-500 dark:text-slate-400 mb-6 max-w-xs">The pure efficiency management system for professional dry cleaners and laundry networks.</p>
        <div class="flex gap-4">
          <!-- Social placeholders -->
          <a class="w-8 h-8 rounded-full bg-slate-200 dark:bg-slate-800 flex items-center justify-center text-slate-600 hover:text-[#2F80ED] transition-colors" href="#"><span class="material-symbols-outlined text-sm" data-icon="share">share</span></a>
          <a class="w-8 h-8 rounded-full bg-slate-200 dark:bg-slate-800 flex items-center justify-center text-slate-600 hover:text-[#2F80ED] transition-colors" href="#"><span class="material-symbols-outlined text-sm" data-icon="mail">mail</span></a>
        </div>
      </div>
      <div>
        <h4 class="font-bold text-slate-900 dark:text-white mb-4 text-sm uppercase tracking-wider">Product</h4>
        <ul class="space-y-3">
          <li><a class="text-sm font-['Inter'] leading-relaxed text-slate-500 dark:text-slate-400 hover:text-[#2F80ED] hover:underline transition-all focus:ring-2 focus:ring-[#2F80ED] outline-none" href="#">Features</a></li>
          <li><a class="text-sm font-['Inter'] leading-relaxed text-slate-500 dark:text-slate-400 hover:text-[#2F80ED] hover:underline transition-all focus:ring-2 focus:ring-[#2F80ED] outline-none" href="#">Pricing</a></li>
          <li><a class="text-sm font-['Inter'] leading-relaxed text-[#2F80ED] font-bold hover:underline transition-all focus:ring-2 focus:ring-[#2F80ED] outline-none" href="#">Role Management</a></li>
          <li><a class="text-sm font-['Inter'] leading-relaxed text-slate-500 dark:text-slate-400 hover:text-[#2F80ED] hover:underline transition-all focus:ring-2 focus:ring-[#2F80ED] outline-none" href="#">POS Guide</a></li>
        </ul>
      </div>
      <div>
        <h4 class="font-bold text-slate-900 dark:text-white mb-4 text-sm uppercase tracking-wider">Company</h4>
        <ul class="space-y-3">
          <li><NuxtLink to="/about" class="text-sm font-['Inter'] leading-relaxed text-slate-500 dark:text-slate-400 hover:text-[#2F80ED] hover:underline transition-all focus:ring-2 focus:ring-[#2F80ED] outline-none">About Us</NuxtLink></li>
          <li><NuxtLink to="/how-work" class="text-sm font-['Inter'] leading-relaxed text-slate-500 dark:text-slate-400 hover:text-[#2F80ED] hover:underline transition-all focus:ring-2 focus:ring-[#2F80ED] outline-none">How It Works</NuxtLink></li>
          <li><NuxtLink to="/contact" class="text-sm font-['Inter'] leading-relaxed text-slate-500 dark:text-slate-400 hover:text-[#2F80ED] hover:underline transition-all focus:ring-2 focus:ring-[#2F80ED] outline-none">Contact</NuxtLink></li>
        </ul>
      </div>
      <div>
        <h4 class="font-bold text-slate-900 dark:text-white mb-4 text-sm uppercase tracking-wider">Legal</h4>
        <ul class="space-y-3">
          <li><NuxtLink to="/privacy" class="text-sm font-['Inter'] leading-relaxed text-slate-500 dark:text-slate-400 hover:text-[#2F80ED] hover:underline transition-all focus:ring-2 focus:ring-[#2F80ED] outline-none">Privacy</NuxtLink></li>
          <li><NuxtLink to="/terms" class="text-sm font-['Inter'] leading-relaxed text-slate-500 dark:text-slate-400 hover:text-[#2F80ED] hover:underline transition-all focus:ring-2 focus:ring-[#2F80ED] outline-none">Terms of Service</NuxtLink></li>
        </ul>
      </div>
    </div>
    <div class="max-w-[1440px] mx-auto pt-8 border-t border-slate-200 dark:border-slate-800 text-center md:text-left">
      <p class="text-sm font-['Inter'] leading-relaxed text-slate-500 dark:text-slate-400">© 2024 iLaundry Professional Management. All rights reserved.</p>
    </div>
  </footer>
</template>