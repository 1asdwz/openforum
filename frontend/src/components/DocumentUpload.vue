<template>
  <div class="doc-upload">
    <div class="doc-upload-actions">
      <input
        ref="fileInput"
        type="file"
        accept="image/jpeg,image/png,image/webp"
        multiple
        style="display: none"
        @change="onPick"
      />
      <el-button :disabled="disabled || full" @click="fileInput?.click()">
        Select / capture document photos
      </el-button>
      <span class="doc-upload-count">
        Uploaded {{ items.length }} / {{ maxCount }}
      </span>
    </div>

    <el-alert
      v-if="hint"
      :title="hint"
      type="info"
      :closable="false"
      show-icon
      style="margin: 10px 0"
    />

    <div v-if="items.length" class="doc-upload-grid">
      <div v-for="item in items" :key="item.mediaId" class="doc-upload-item">
        <img :src="item.url" :alt="item.name" />
        <div class="doc-upload-item-mask">
          <el-button size="small" type="danger" @click="remove(item.mediaId)">
            Delete
          </el-button>
        </div>
        <div class="doc-upload-item-name" :title="item.name">{{ item.name }}</div>
      </div>
    </div>
    <div v-else class="doc-upload-empty">No document photos uploaded yet</div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { authenticationApi } from '@/api'
import { errorMessage } from '@/utils/format'

interface UploadItem {
  mediaId: number
  name: string
  url: string
  sizeBytes: number
}

const props = withDefaults(
  defineProps<{
    minCount?: number
    maxCount?: number
    disabled?: boolean
  }>(),
  { minCount: 1, maxCount: 5, disabled: false }
)

const emit = defineEmits<{ (e: 'change', mediaIds: number[]): void }>()

const fileInput = ref<HTMLInputElement | null>(null)
const items = ref<UploadItem[]>([])
const uploading = ref(false)

const full = computed(() => items.value.length >= props.maxCount)

const hint = computed(() => {
  if (props.disabled) return ''
  if (items.value.length < props.minCount) {
    return `Upload at least ${props.minCount} clear document photo(s) (JPEG/PNG/WEBP, max 10MB each)`
  }
  return 'Photos must be complete and clear, with no glare or obstruction and all four corners in frame'
})

function onPick(e: Event) {
  const input = e.target as HTMLInputElement
  const files = Array.from(input.files || [])
  input.value = ''
  if (!files.length) return
  void upload(files)
}

async function upload(files: File[]) {
  if (props.disabled) return
  const remain = props.maxCount - items.value.length
  if (remain <= 0) {
    ElMessage.warning(`You can upload at most ${props.maxCount} photos`)
    return
  }
  uploading.value = true
  try {
    for (const file of files.slice(0, remain)) {
      if (!/^image\/(jpeg|png|webp)$/i.test(file.type)) {
        ElMessage.warning(`"${file.name}" has an unsupported format; please upload JPEG/PNG/WEBP`)
        continue
      }
      if (file.size > 10 * 1024 * 1024) {
        ElMessage.warning(`"${file.name}" exceeds 10MB`)
        continue
      }
      try {
        const res = await authenticationApi.uploadMedia(file, 'DOCUMENT')
        items.value.push({
          mediaId: res.mediaId,
          name: file.name,
          url: URL.createObjectURL(file),
          sizeBytes: res.sizeBytes ?? file.size,
        })
        emit('change', items.value.map((i) => i.mediaId))
      } catch (err) {
        ElMessage.error(errorMessage(err, `Failed to upload "${file.name}"`))
      }
    }
  } finally {
    uploading.value = false
  }
}

function remove(mediaId: number) {
  const idx = items.value.findIndex((i) => i.mediaId === mediaId)
  if (idx < 0) return
  const [removed] = items.value.splice(idx, 1)
  if (removed?.url) URL.revokeObjectURL(removed.url)
  emit('change', items.value.map((i) => i.mediaId))
}

/** Expose the currently uploaded media IDs to the parent component */
function getMediaIds(): number[] {
  return items.value.map((i) => i.mediaId)
}

defineExpose({ getMediaIds })

onBeforeUnmount(() => {
  items.value.forEach((i) => i.url && URL.revokeObjectURL(i.url))
})
</script>

<style scoped>
.doc-upload-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}
.doc-upload-count {
  color: var(--of-text-muted);
  font-size: 13px;
}
.doc-upload-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 12px;
  margin-top: 12px;
}
.doc-upload-item {
  position: relative;
  border: 1px solid var(--el-border-color, #dcdfe6);
  border-radius: 4px;
  overflow: hidden;
  background: #f5f7fa;
}
.doc-upload-item img {
  width: 100%;
  height: 110px;
  object-fit: cover;
  display: block;
}
.doc-upload-item-mask {
  position: absolute;
  top: 4px;
  right: 4px;
}
.doc-upload-item-name {
  font-size: 12px;
  color: var(--of-text-muted);
  padding: 4px 6px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.doc-upload-empty {
  margin-top: 12px;
  color: var(--of-text-muted);
  font-size: 13px;
}
</style>
