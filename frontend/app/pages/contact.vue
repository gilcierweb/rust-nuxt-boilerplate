<template>
  <div>
    <main class="pt-20">
      <!-- Hero Section -->
      <section class="relative bg-surface-container-lowest overflow-hidden py-24 lg:py-32">
        <div class="absolute inset-0 bg-gradient-to-br from-primary-container/10 to-transparent pointer-events-none"></div>
        <div class="max-w-[1440px] mx-auto px-6 text-center relative z-10">
          <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-primary-container/10 text-primary-fixed-variant text-label-sm font-label-sm mb-6 border border-primary-container/20">
            <span class="material-symbols-outlined text-sm" data-icon="mail">mail</span>
            Get in Touch
          </div>
          <h1 class="font-display-xl text-display-xl text-on-surface mb-6 leading-tight max-w-4xl mx-auto">
            We'd Love to Hear from You
          </h1>
          <p class="font-body-lg text-body-lg text-on-surface-variant mb-10 max-w-2xl mx-auto">
            Have questions about the boilerplate? Need enterprise support? Want to contribute? Our team is here to help.
          </p>
        </div>
      </section>

      <!-- Contact Grid -->
      <section class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid lg:grid-cols-3 gap-6 lg:gap-8">
            <!-- Contact Info Cards -->
            <div class="lg:col-span-1 space-y-6">
              <!-- Email -->
              <div class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30">
                <div class="w-12 h-12 rounded-xl bg-primary-container/10 flex items-center justify-center mb-4 text-primary">
                  <span class="material-symbols-outlined text-2xl" data-icon="email">email</span>
                </div>
                <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Email</h3>
                <p class="font-body-md text-body-md text-on-surface-variant mb-3">
                  Response within 24 hours
                </p>
                <a href="mailto:hello@boilerplate.dev" class="text-primary font-medium hover:underline">
                  hello@boilerplate.dev
                </a>
              </div>

              <!-- GitHub -->
              <div class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30">
                <div class="w-12 h-12 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-4 text-secondary">
                  <span class="material-symbols-outlined text-2xl" data-icon="code">code</span>
                </div>
                <h3 class="font-headline-md text-headline-md text-on-surface mb-2">GitHub</h3>
                <p class="font-body-md text-body-md text-on-surface-variant mb-3">
                  Issues, discussions, contributions
                </p>
                <a href="https://github.com/your-org/rust-nuxt-boilerplate" target="_blank" class="text-secondary font-medium hover:underline flex items-center gap-1">
                  <span class="material-symbols-outlined text-sm" data-icon="open_in_new">open_in_new</span>
                  View Repository
                </a>
              </div>

              <!-- Discord -->
              <div class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30">
                <div class="w-12 h-12 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-4 text-tertiary">
                  <span class="material-symbols-outlined text-2xl" data-icon="chat">chat</span>
                </div>
                <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Discord Community</h3>
                <p class="font-body-md text-body-md text-on-surface-variant mb-3">
                  Real-time help and discussion
                </p>
                <a href="https://discord.gg/your-invite" target="_blank" class="text-tertiary font-medium hover:underline flex items-center gap-1">
                  <span class="material-symbols-outlined text-sm" data-icon="open_in_new">open_in_new</span>
                  Join Server
                </a>
              </div>
            </div>

            <!-- Contact Form -->
            <div class="lg:col-span-2">
              <div class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30">
                <h2 class="font-headline-lg text-headline-lg text-on-surface mb-6">
                  Send a Message
                </h2>
                
                <form @submit.prevent="handleSubmit" class="space-y-6">
                  <div class="grid md:grid-cols-2 gap-6">
                    <!-- Name -->
                    <div class="form-control">
                      <label class="label">
                        <span class="label-text font-medium text-on-surface">Full Name</span>
                        <span class="text-error">*</span>
                      </label>
                      <input
                        v-model="form.name"
                        type="text"
                        placeholder="Your name"
                        class="input input-bordered w-full bg-surface"
                        required
                      />
                    </div>

                    <!-- Email -->
                    <div class="form-control">
                      <label class="label">
                        <span class="label-text font-medium text-on-surface">Email</span>
                        <span class="text-error">*</span>
                      </label>
                      <input
                        v-model="form.email"
                        type="email"
                        placeholder="you@example.com"
                        class="input input-bordered w-full bg-surface"
                        required
                      />
                    </div>
                  </div>

                  <div class="grid md:grid-cols-2 gap-6">
                    <!-- Company/Org (optional) -->
                    <div class="form-control">
                      <label class="label">
                        <span class="label-text font-medium text-on-surface">Company / Organization</span>
                      </label>
                      <input
                        v-model="form.company"
                        type="text"
                        placeholder="Your company (optional)"
                        class="input input-bordered w-full bg-surface"
                      />
                    </div>

                    <!-- Subject -->
                    <div class="form-control">
                      <label class="label">
                        <span class="label-text font-medium text-on-surface">Subject</span>
                        <span class="text-error">*</span>
                      </label>
                      <select v-model="form.subject" class="select select-bordered w-full bg-surface" required>
                        <option value="">Select a topic...</option>
                        <option value="general">General Inquiry</option>
                        <option value="support">Technical Support</option>
                        <option value="enterprise">Enterprise / Custom Features</option>
                        <option value="security">Security Issue</option>
                        <option value="contribution">Contribution / Partnership</option>
                        <option value="other">Other</option>
                      </select>
                    </div>
                  </div>

                  <!-- Message -->
                  <div class="form-control">
                    <label class="label">
                      <span class="label-text font-medium text-on-surface">Message</span>
                      <span class="text-error">*</span>
                    </label>
                    <textarea
                      v-model="form.message"
                      placeholder="How can we help you?"
                      class="textarea textarea-bordered w-full bg-surface min-h-[150px]"
                      required
                    ></textarea>
                  </div>

                  <!-- Privacy Checkbox -->
                  <div class="form-control">
                    <label class="label cursor-pointer justify-start gap-3">
                      <input v-model="form.privacy" type="checkbox" class="checkbox checkbox-primary" required />
                      <span class="label-text text-on-surface-variant">
                        I agree to the <NuxtLink to="/privacy" class="text-primary hover:underline">Privacy Policy</NuxtLink> and understand my data will be used to respond to this inquiry.
                      </span>
                    </label>
                  </div>

                  <!-- Submit -->
                  <div class="flex items-center gap-4 pt-4">
                    <button
                      type="submit"
                      class="btn btn-primary px-8"
                      :disabled="submitting"
                    >
                      <span v-if="submitting" class="loading loading-spinner loading-sm mr-2"></span>
                      <span v-else class="material-symbols-outlined mr-2" data-icon="send">send</span>
                      {{ submitting ? 'Sending...' : 'Send Message' }}
                    </button>
                    
                    <span v-if="success" class="text-success flex items-center gap-2">
                      <span class="material-symbols-outlined" data-icon="check_circle">check_circle</span>
                      Message sent successfully! We'll get back to you soon.
                    </span>
                    
                    <span v-if="error" class="text-error flex items-center gap-2">
                      <span class="material-symbols-outlined" data-icon="error">error</span>
                      Failed to send. Please try again or email us directly.
                    </span>
                  </div>
                </form>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- FAQ Section -->
      <section class="py-24 bg-surface-container-low border-y border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-12">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">Frequently Asked Questions</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
              Quick answers to common questions about the boilerplate.
            </p>
          </div>

          <div class="grid md:grid-cols-2 gap-6 max-w-4xl mx-auto">
            <!-- FAQ 1 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="material-symbols-outlined text-primary" data-icon="help">help</span>
                Is this production-ready?
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Yes! The boilerplate includes authentication, authorization, admin panel, database migrations, Docker setup, and comprehensive tests. It's designed for production use from day one.
              </p>
            </div>

            <!-- FAQ 2 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="material-symbols-outlined text-secondary" data-icon="help">help</span>
                Can I use this for commercial projects?
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Absolutely! The project is MIT licensed, which permits commercial use, modification, distribution, and private use without attribution requirements.
              </p>
            </div>

            <!-- FAQ 3 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="material-symbols-outlined text-tertiary" data-icon="help">help</span>
                How do I get support?
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Community support is available via GitHub Discussions and Discord. For enterprise customers, we offer dedicated support channels with SLA guarantees and priority response times.
              </p>
            </div>

            <!-- FAQ 4 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="material-symbols-outlined text-success" data-icon="help">help</span>
                Does it work with my existing database?
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Yes! The boilerplate uses Diesel ORM with PostgreSQL. You can connect to any PostgreSQL-compatible database (AWS RDS, Supabase, Neon, Cloud SQL, etc.) by updating the DATABASE_URL environment variable.
              </p>
            </div>

            <!-- FAQ 5 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="material-symbols-outlined text-warning" data-icon="help">help</span>
                Can I remove features I don't need?
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Yes! The modular architecture makes it easy to remove unused modules. Each feature (auth, admin, websockets, etc.) is isolated. Simply remove the routes, controllers, and dependencies you don't need.
              </p>
            </div>

            <!-- FAQ 6 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="material-symbols-outlined text-info" data-icon="help">help</span>
                How do I deploy this?
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                The boilerplate includes Docker multi-stage builds and docker-compose for local development. For production, deploy the Docker images to any container platform (Kubernetes, ECS, Cloud Run, Fly.io, Railway, etc.). Health checks and graceful shutdown are built-in.
              </p>
            </div>
          </div>
        </div>
      </section>

      <!-- Support Section -->
      <section class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="bg-surface p-8 md:p-12 rounded-3xl border border-outline-variant/30 text-center relative overflow-hidden">
            <div class="absolute -right-20 -top-20 w-64 h-64 bg-primary/5 rounded-full blur-3xl"></div>
            <div class="absolute -left-20 -bottom-20 w-64 h-64 bg-secondary/5 rounded-full blur-3xl"></div>
            
            <div class="relative z-10">
              <h2 class="font-display-xl text-display-xl text-on-surface mb-4">Need Enterprise Support?</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8 max-w-2xl mx-auto">
                Dedicated support, custom feature development, security audits, migration assistance, and SLA-backed response times.
              </p>
              <div class="flex flex-wrap justify-center gap-4">
                <NuxtLink to="/contact" class="bg-primary text-on-primary px-8 py-4 rounded-xl font-headline-md text-headline-md font-medium hover:bg-primary-container hover:text-on-primary-container hover:-translate-y-0.5 active:scale-95 transition-all shadow-sm shadow-primary/20 flex items-center gap-2">
                  <span class="material-symbols-outlined" data-icon="business_center">business_center</span>
                  Contact Sales
                </NuxtLink>
                <NuxtLink to="https://github.com/your-org/rust-nuxt-boilerplate/discussions" target="_blank" class="px-8 py-4 rounded-xl font-headline-md text-headline-md text-on-surface hover:bg-surface-container transition-colors flex items-center gap-2 border border-outline-variant/50">
                  <span class="material-symbols-outlined" data-icon="forum">forum</span>
                  Community Forum
                </NuxtLink>
              </div>
            </div>
          </div>
        </div>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  layout: 'landing'
})

const form = reactive({
  name: '',
  email: '',
  company: '',
  subject: '',
  message: '',
  privacy: false
})

const submitting = ref(false)
const success = ref(false)
const error = ref(false)

async function handleSubmit() {
  submitting.value = true
  error.value = false
  
  // Simulate API call
  await new Promise(resolve => setTimeout(resolve, 1500))
  
  submitting.value = false
  success.value = true
  
  // Reset form
  Object.keys(form).forEach(key => {
    form[key] = key === 'privacy' ? false : ''
  })
  
  // Reset success message after 5 seconds
  setTimeout(() => {
    success.value = false
  }, 5000)
}
</script>