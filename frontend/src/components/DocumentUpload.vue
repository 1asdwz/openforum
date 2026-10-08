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
        选择 / 拍摄证件照片
      </el-button>
      <span class="doc-upload-count">
        已上传 {{ items.length }} / {{ maxCount }} 张
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
            删除
          </el-button>
        </div>
        <div class="doc-upload-item-name" :title="item.name">{{ item.name }}</div>
      </div>
    </div>
    <div v-else class="doc-upload-empty">尚未上传证件照片</div>
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
    return `请上传至少 ${props.minCount} 张清晰证件照片（支持 JPEG/PNG/WEBP，单张不超过 10MB）`
  }
  return '照片需完整、清晰、无反光、无遮挡，四角完整入镜'
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
    ElMessage.warning(`最多上传 ${props.maxCount} 张`)
    return
  }
  uploading.value = true
  try {
    for (const file of files.slice(0, remain)) {
      if (!/^image\/(jpeg|png|webp)$/i.test(file.type)) {
        ElMessage.warning(`「${file.name}」格式不支持，请上传 JPEG/PNG/WEBP`)
        continue
      }
      if (file.size > 10 * 1024 * 1024) {
        ElMessage.warning(`「${file.name}」超过 10MB`)
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
        ElMessage.error(errorMessage(err, `「${file.name}」上传失败`))
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

/** 供父组件读取当前已上传的媒体 id */
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
