# OpenForum 开发贡献指南

## 代码规范

### 后端 (Java)

#### 命名规范

- 类名：PascalCase (例：`UserService`)
- 方法名：camelCase (例：`getUserById`)
- 常量：UPPER_SNAKE_CASE (例：`DEFAULT_PAGE_SIZE`)
- 包名：全小写，按功能分类 (例：`com.openforum.service`)

#### 代码风格

```java
// ✅ Good
@Service
@RequiredArgsConstructor
public class UserService {
    private final UserRepository userRepository;
    
    public User getUserById(Long id) {
        return userRepository.findById(id)
            .orElseThrow(() -> new ApiException.NotFoundException("用户不存在"));
    }
}

// ❌ Bad
public class userService {
    UserRepository userRepository;
    
    public User getUserById(Long id) {
        return userRepository.findById(id).get();
    }
}
```

#### 注释规范

```java
/**
 * 获取用户信息
 * 
 * @param userId 用户ID
 * @return 用户对象
 * @throws NotFoundException 用户不存在时抛出
 */
@Transactional(readOnly = true)
public User getUserById(Long userId) {
    // ...
}
```

### 前端 (Vue)

#### 命名规范

- 组件名：PascalCase (例：`NavBar.vue`)
- 方法名：camelCase (例：`handleSubmit`)
- 常量：UPPER_SNAKE_CASE (例：`API_BASE_URL`)

#### 代码风格

```vue
<!-- ✅ Good -->
<template>
  <div class="user-card">
    <h1>{{ user.name }}</h1>
    <el-button @click="handleSubmit">提交</el-button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

interface User {
  id: number
  name: string
}

const user = ref<User | null>(null)
const loading = ref(false)

const isLoading = computed(() => loading.value)

const handleSubmit = () => {
  // ...
}
</script>

<!-- ❌ Bad -->
<template>
  <div>
    <h1>{{ userData.name }}</h1>
    <button @click="submit">提交</button>
  </div>
</template>

<script>
export default {
  data() {
    return {
      userData: null
    }
  },
  methods: {
    submit() {
      // ...
    }
  }
}
</script>
```

## 提交流程

### 1. Fork 项目

```bash
git clone https://github.com/your-username/openforum.git
cd openforum
```

### 2. 创建特性分支

```bash
git checkout -b feature/your-feature-name
```

分支命名规范：
- `feature/` - 新功能
- `bugfix/` - bug 修复
- `docs/` - 文档
- `refactor/` - 代码重构

### 3. 提交代码

```bash
git add .
git commit -m "feat: 添加用户认证功能

- 实现 JWT token 生成
- 添加认证中间件
- 完成单元测试"
```

提交信息格式：
```
<type>: <subject>

<body>

<footer>
```

Type 类型：
- `feat:` - 新功能
- `fix:` - bug 修复
- `docs:` - 文档更新
- `style:` - 代码格式调整
- `refactor:` - 代码重构
- `perf:` - 性能优化
- `test:` - 测试相关

### 4. 推送分支

```bash
git push origin feature/your-feature-name
```

### 5. 创建 Pull Request

在 GitHub 创建 PR，说明：
- 功能描述
- 变更内容
- 测试方式
- 相关 Issue

## 测试

### 后端测试

```bash
cd backend

# 运行所有测试
mvn test

# 运行特定测试
mvn test -Dtest=UserServiceTest
```

### 前端测试

```bash
cd frontend

# 运行测试
npm run test

# 测试覆盖率
npm run test:coverage
```

## 代码审查清单

提交 PR 前请确认：

- [ ] 代码遵循项目规范
- [ ] 添加了必要的注释
- [ ] 代码已测试
- [ ] 没有硬编码配置
- [ ] 没有调试代码
- [ ] 文档已更新
- [ ] 提交信息清晰

## 报告 Issue

### Bug 报告

```markdown
## 描述
清晰地描述 bug

## 重现步骤
1. 打开页面
2. 点击按钮
3. 观察结果

## 预期行为
应该发生什么

## 实际行为
实际发生了什么

## 环境
- OS: Windows 10
- Browser: Chrome 120
- Version: v0.1.0
```

### 功能请求

```markdown
## 功能描述
新功能的简要说明

## 使用场景
为什么需要这个功能

## 可能的实现方案
如果你有想法
```

## 文档贡献

文档位置：
- README.md - 项目概览
- QUICK_START.md - 快速开始
- 各模块 README.md - 模块文档
- docs/ - 详细文档

文档格式：
- 使用 Markdown
- 包含示例代码
- 清晰的章节结构

## 许可证

所有贡献均采用 MIT 许可证。

---

感谢你的贡献！🎉
