<template>
  <el-dialog v-model="visible" :show-close="false" align-center class="about-dialog" width="560px">
    <div :key="visible ? 'open' : 'closed'" class="about-container">
      <!-- Close button -->
      <button class="close-btn" @click="visible = false">
        <svg fill="none" height="20" viewBox="0 0 20 20" width="20">
          <path d="M15 5L5 15M5 5L15 15" stroke="currentColor" stroke-linecap="round" stroke-width="1.5" />
        </svg>
      </button>

      <!-- Decorative data grid background -->
      <div class="grid-overlay"></div>

      <!-- Logo and branding -->
      <div class="brand-section">
        <div class="logo-container">
          <svg class="logo" fill="none" height="64" viewBox="0 0 64 64" width="64">
            <!-- Geometric data visualization icon -->
            <rect fill="url(#grad1)" height="32" opacity="0.9" rx="2" width="8" x="4" y="28" />
            <rect fill="url(#grad1)" height="40" opacity="0.95" rx="2" width="8" x="16" y="20" />
            <rect fill="url(#grad2)" height="48" rx="2" width="8" x="28" y="12" />
            <rect fill="url(#grad1)" height="36" opacity="0.95" rx="2" width="8" x="40" y="24" />
            <rect fill="url(#grad1)" height="44" opacity="0.9" rx="2" width="8" x="52" y="16" />

            <!-- Connecting line -->
            <path
              d="M8 44 L20 36 L32 20 L44 38 L56 28"
              opacity="0.6"
              stroke="url(#lineGrad)"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
            />

            <defs>
              <linearGradient id="grad1" x1="0" x2="0" y1="0" y2="1">
                <stop offset="0%" stop-color="#667eea" />
                <stop offset="100%" stop-color="#764ba2" />
              </linearGradient>
              <linearGradient id="grad2" x1="0" x2="0" y1="0" y2="1">
                <stop offset="0%" stop-color="#f093fb" />
                <stop offset="100%" stop-color="#f5576c" />
              </linearGradient>
              <linearGradient id="lineGrad" x1="0" x2="1" y1="0" y2="0">
                <stop offset="0%" stop-color="#667eea" />
                <stop offset="100%" stop-color="#f5576c" />
              </linearGradient>
            </defs>
          </svg>
        </div>

        <h1 class="app-name">DataVis Pro</h1>
        <p class="phonetic">/ˈdeɪtə vɪz proʊ/</p>
        <p class="tagline">Professional Data Analysis & Visualization</p>
        <div class="version">Version {{ appVersion }}</div>
      </div>

      <!-- Divider with data point -->
      <div class="divider">
        <div class="data-point"></div>
      </div>

      <!-- Tech stack -->
      <div class="tech-section">
        <div class="tech-label">Built with</div>
        <div class="tech-grid">
          <div
            v-for="(tech, index) in technologies"
            :key="tech.name"
            :style="{ animationDelay: `${index * 0.1}s` }"
            class="tech-item"
          >
            <div :style="{ background: tech.color }" class="tech-icon">
              <span>{{ tech.icon }}</span>
            </div>
            <div class="tech-info">
              <div class="tech-name">{{ tech.name }}</div>
              <div class="tech-desc">{{ tech.description }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="footer">
        <div class="copyright">© 2026 DataVis Pro</div>
        <div class="credits">Powered by ApplePine</div>
      </div>
    </div>
  </el-dialog>
</template>

<script lang="ts" setup>
import { getVersion } from '@tauri-apps/api/app';
import { onMounted, ref } from 'vue';

const visible = defineModel<boolean>({ required: true });
const appVersion = ref('1.0.0');

const technologies = [
  {
    name: 'Tauri',
    description: 'Desktop Framework',
    icon: '⚡',
    color: 'linear-gradient(135deg, #FFC107 0%, #FF6F00 100%)',
  },
  {
    name: 'Rust',
    description: 'Backend Engine',
    icon: '⚙️',
    color: 'linear-gradient(135deg, #CE422B 0%, #8B0000 100%)',
  },
  {
    name: 'Vue 3',
    description: 'UI Framework',
    icon: '💚',
    color: 'linear-gradient(135deg, #42b883 0%, #35495e 100%)',
  },
  {
    name: 'Polars',
    description: 'Data Processing',
    icon: '🐻‍❄️',
    color: 'linear-gradient(135deg, #00D9FF 0%, #0066CC 100%)',
  },
];

onMounted(async () => {
  try {
    appVersion.value = await getVersion();
  } catch (error) {
    console.error('Failed to get app version:', error);
  }
});
</script>

<style scoped>
.about-dialog :deep(.el-dialog) {
  background: linear-gradient(135deg, #f8f9fa 0%, #ffffff 100%);
  border-radius: 24px;
  padding: 0;
  overflow: hidden;
  box-shadow:
    0 25px 50px -12px rgba(0, 0, 0, 0.25),
    0 0 0 1px rgba(0, 0, 0, 0.05);
}

.about-dialog :deep(.el-dialog__header) {
  display: none;
}

.about-dialog :deep(.el-dialog__body) {
  padding: 0;
}

.about-container {
  position: relative;
  padding: 48px 40px 32px;
  color: #1f2937;
}

/* Decorative grid background */
.grid-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-image:
    linear-gradient(rgba(102, 126, 234, 0.06) 1px, transparent 1px),
    linear-gradient(90deg, rgba(102, 126, 234, 0.06) 1px, transparent 1px);
  background-size: 20px 20px;
  pointer-events: none;
  opacity: 0;
  animation: gridFadeIn 1s ease-out 0.3s forwards;
}

@keyframes gridFadeIn {
  to {
    opacity: 1;
  }
}

/* Close button */
.close-btn {
  position: absolute;
  top: 20px;
  right: 20px;
  width: 36px;
  height: 36px;
  border: none;
  background: rgba(0, 0, 0, 0.05);
  border-radius: 8px;
  color: #6b7280;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  z-index: 10;
}

.close-btn:hover {
  background: rgba(0, 0, 0, 0.1);
  color: #1f2937;
  transform: rotate(90deg);
}

/* Brand section */
.brand-section {
  text-align: center;
  margin-bottom: 40px;
  animation: slideUp 0.6s ease-out;
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.logo-container {
  margin-bottom: 24px;
  animation: logoFloat 3s ease-in-out infinite;
}

@keyframes logoFloat {
  0%,
  100% {
    transform: translateY(0px);
  }
  50% {
    transform: translateY(-8px);
  }
}

.logo {
  filter: drop-shadow(0 8px 16px rgba(102, 126, 234, 0.3));
}

.app-name {
  font-family:
    'SF Pro Display',
    -apple-system,
    BlinkMacSystemFont,
    sans-serif;
  font-size: 36px;
  font-weight: 700;
  letter-spacing: -0.02em;
  margin: 0 0 4px 0;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.phonetic {
  font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  color: #6b7280;
  margin: 0 0 12px 0;
  letter-spacing: 0.02em;
}

.tagline {
  font-family:
    'SF Pro Text',
    -apple-system,
    BlinkMacSystemFont,
    sans-serif;
  font-size: 15px;
  color: #4b5563;
  margin: 0 0 16px 0;
  font-weight: 400;
  letter-spacing: 0.01em;
}

.version {
  display: inline-block;
  padding: 6px 16px;
  background: rgba(102, 126, 234, 0.1);
  border: 1px solid rgba(102, 126, 234, 0.3);
  border-radius: 20px;
  font-family: 'SF Mono', 'Menlo', monospace;
  font-size: 13px;
  color: #5b21b6;
  font-weight: 500;
  letter-spacing: 0.02em;
}

/* Divider */
.divider {
  position: relative;
  height: 1px;
  background: linear-gradient(90deg, transparent 0%, rgba(102, 126, 234, 0.2) 50%, transparent 100%);
  margin: 32px 0;
}

.data-point {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 8px;
  height: 8px;
  background: linear-gradient(135deg, #667eea 0%, #f5576c 100%);
  border-radius: 50%;
  box-shadow: 0 0 12px rgba(102, 126, 234, 0.6);
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%,
  100% {
    transform: translate(-50%, -50%) scale(1);
    opacity: 1;
  }
  50% {
    transform: translate(-50%, -50%) scale(1.3);
    opacity: 0.7;
  }
}

/* Tech section */
.tech-section {
  margin-bottom: 32px;
}

.tech-label {
  font-family:
    'SF Pro Text',
    -apple-system,
    BlinkMacSystemFont,
    sans-serif;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: #6b7280;
  margin-bottom: 16px;
  font-weight: 600;
}

.tech-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.tech-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px;
  background: #ffffff;
  border: 1px solid #e5e7eb;
  border-radius: 12px;
  transition: all 0.3s ease;
  opacity: 0;
  animation: techItemFadeIn 0.5s ease-out forwards;
}

@keyframes techItemFadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.tech-item:hover {
  background: #f9fafb;
  border-color: rgba(102, 126, 234, 0.4);
  transform: translateY(-2px);
  box-shadow: 0 8px 16px rgba(102, 126, 234, 0.15);
}

.tech-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  flex-shrink: 0;
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
}

.tech-info {
  flex: 1;
  min-width: 0;
}

.tech-name {
  font-family:
    'SF Pro Text',
    -apple-system,
    BlinkMacSystemFont,
    sans-serif;
  font-size: 14px;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 2px;
  letter-spacing: -0.01em;
}

.tech-desc {
  font-family:
    'SF Pro Text',
    -apple-system,
    BlinkMacSystemFont,
    sans-serif;
  font-size: 12px;
  color: #6b7280;
  letter-spacing: 0.01em;
}

/* Footer */
.footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 24px;
  border-top: 1px solid #e5e7eb;
  font-family:
    'SF Pro Text',
    -apple-system,
    BlinkMacSystemFont,
    sans-serif;
  font-size: 12px;
  color: #6b7280;
  animation: fadeIn 0.6s ease-out 0.4s both;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.copyright {
  letter-spacing: 0.01em;
}

.credits {
  letter-spacing: 0.01em;
}

/* Responsive adjustments */
@media (max-width: 600px) {
  .about-container {
    padding: 40px 24px 24px;
  }

  .app-name {
    font-size: 28px;
  }

  .tech-grid {
    grid-template-columns: 1fr;
  }
}
</style>
