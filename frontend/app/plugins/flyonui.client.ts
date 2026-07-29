import { useRouter } from "vue-router";

// Optional third-party libraries
// import $ from "jquery";
// import _ from "lodash";
// import noUiSlider from "nouislider";
// import "datatables.net";
// import "dropzone/dist/dropzone-min.js";

// window._ = _;
// window.$ = $;
// window.jQuery = $;
// window.DataTable = $.fn.dataTable;
// window.noUiSlider = noUiSlider;

// FlyonUI
import "flyonui/flyonui";

export default defineNuxtPlugin(() => {
  const router = useRouter();

  router.beforeEach((to) => {
    // Block any navigation containing "undefined" as a path segment
    // This prevents spurious XHR requests from NuxtLink prefetch or other sources
    if (/\/undefined(\/|$)/.test(to.path)) {
      const segments = to.path.split('/').filter(Boolean)
      const undefinedIndex = segments.findIndex((s) => s === 'undefined')

      // Extract resource name dynamically (segment before "undefined")
      let redirectPath = '/admin/dashboard'
      if (undefinedIndex > 0) {
        const resource = segments[undefinedIndex - 1]
        if (resource !== 'admin') {
          redirectPath = `/admin/${resource}`
        }
      }

      return redirectPath
    }
  })

  // setTimeout (macro-task) is intentional: it defers autoInit until after the
  // browser has finished painting the new page DOM. nextTick/microtasks run too
  // early — the DOM is not yet stable when FlyonUI tries to bind event listeners.
  // This matches the official FlyonUI Nuxt integration guide exactly.
  router.afterEach(() => {
    setTimeout(() => window.HSStaticMethods.autoInit())
  })
})
