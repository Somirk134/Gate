<template>
  <section class="servers-page">
    <header class="server-page-header">
      <div>
        <p>Runtime Servers</p>
        <h1>服务器</h1>
        <span>{{ onlineServers.length }} 台已连接 / {{ servers.length }} 台已保存</span>
      </div>
      <div class="server-page-header__actions">
        <GButton
          variant="secondary"
          icon="refresh"
          :loading="isLoading"
          @click="refresh"
        >
          刷新
        </GButton>
        <GButton
          variant="primary"
          icon="plus"
          @click="openCreate"
        >
          添加服务器
        </GButton>
      </div>
    </header>

    <GCard
      v-if="isError"
      variant="plain"
      padding="lg"
    >
      <GErrorState
        title="服务器加载失败"
        :message="error"
        retry
        @retry="retry"
      />
    </GCard>

    <div
      v-else-if="!hasServers && !isLoading"
      class="server-empty"
    >
      <div class="server-empty__icon">
        <GIcon
          name="servers"
          :size="34"
        />
      </div>
      <h2>先连接服务器，再创建隧道</h2>
      <p>Gate Tunnel 需要一个已认证的服务端连接。添加服务器地址和 Token 后，点击连接即可开始测试 TCP/HTTP/HTTPS Tunnel。</p>
      <div class="server-empty__actions">
        <GButton
          variant="primary"
          icon="plus"
          @click="openCreate"
        >
          添加第一台服务器
        </GButton>
        <GButton
          variant="secondary"
          icon="activity"
          @click="router.push('/diagnostics')"
        >
          打开诊断
        </GButton>
      </div>
    </div>

    <div
      v-else
      class="server-shell"
    >
      <aside class="server-list-panel">
        <div class="server-list-panel__toolbar">
          <label class="server-search">
            <GIcon
              name="search"
              :size="15"
            />
            <input
              v-model.trim="query"
              placeholder="搜索名称、地址或标签"
            >
          </label>
        </div>

        <div class="server-list-panel__body">
          <button
            v-for="server in filteredServers"
            :key="server.id"
            type="button"
            class="server-list-row"
            :class="{ active: selectedId === server.id }"
            @click="selectedId = server.id"
          >
            <span
              class="server-list-row__status"
              :class="`is-${statusTone(server.status)}`"
            />
            <span class="server-list-row__icon">
              <GIcon
                name="servers"
                :size="16"
              />
            </span>
            <span class="server-list-row__main">
              <strong>{{ server.name }}</strong>
              <small>{{ server.settings.host }}:{{ server.settings.port }}</small>
            </span>
            <span class="server-list-row__meta">
              <strong>{{ statusLabel(server.status) }}</strong>
              <small>{{ server.ping ? `${server.ping}ms` : server.lastConnectedAt }}</small>
            </span>
          </button>

          <div
            v-if="!filteredServers.length"
            class="server-list-empty"
          >
            <GIcon
              name="search"
              :size="24"
            />
            <span>没有匹配的服务器</span>
          </div>
        </div>
      </aside>

      <main class="server-detail-panel">
        <template v-if="selectedServer">
          <div class="server-detail-header">
            <div>
              <div class="server-detail-header__title">
                <span :class="`is-${statusTone(selectedServer.status)}`" />
                <h2>{{ selectedServer.name }}</h2>
              </div>
              <p>{{ selectedServer.settings.host }}:{{ selectedServer.settings.port }}</p>
            </div>
            <div class="server-detail-header__actions">
              <GButton
                v-if="selectedServer.status !== 'connected'"
                variant="primary"
                icon="plug"
                :loading="isConnecting(selectedServer.id)"
                @click="connectSelected"
              >
                连接
              </GButton>
              <GButton
                v-else
                variant="secondary"
                icon="power"
                @click="disconnectSelected"
              >
                断开
              </GButton>
              <GButton
                variant="secondary"
                icon="activity"
                :loading="testingId === selectedServer.id"
                @click="testSelected"
              >
                测试
              </GButton>
              <GButton
                variant="ghost"
                icon="settings"
                @click="openEdit(selectedServer)"
              >
                编辑
              </GButton>
              <GButton
                variant="danger"
                icon="trash"
                @click="removeSelected"
              >
                删除
              </GButton>
            </div>
          </div>

          <div class="server-summary-grid">
            <article>
              <span>连接状态</span>
              <strong>{{ statusLabel(selectedServer.status) }}</strong>
            </article>
            <article>
              <span>认证会话</span>
              <strong>{{ selectedServer.version }}</strong>
            </article>
            <article>
              <span>最近连接</span>
              <strong>{{ selectedServer.lastConnectedAt }}</strong>
            </article>
            <article>
              <span>RTT</span>
              <strong>{{ selectedServer.ping ? `${selectedServer.ping} ms` : "-" }}</strong>
            </article>
          </div>

          <section class="server-info-section">
            <div class="server-info-section__head">
              <h3>连接配置</h3>
              <button
                type="button"
                @click="copyServerInfo(selectedServer)"
              >
                <GIcon
                  name="copy"
                  :size="15"
                />
              </button>
            </div>
            <dl class="server-info-list">
              <div><dt>Host</dt><dd>{{ selectedServer.settings.host }}</dd></div>
              <div><dt>Port</dt><dd>{{ selectedServer.settings.port }}</dd></div>
              <div><dt>Token</dt><dd>{{ maskToken(selectedServer.settings.token) }}</dd></div>
              <div><dt>Kind</dt><dd>{{ selectedServer.kind }}</dd></div>
              <div><dt>Region</dt><dd>{{ selectedServer.region || "-" }}</dd></div>
              <div><dt>Auto Connect</dt><dd>{{ selectedServer.settings.autoConnect ? "On" : "Off" }}</dd></div>
            </dl>
          </section>

          <section class="server-info-section">
            <div class="server-info-section__head">
              <h3>健康状态</h3>
              <span>{{ selectedServer.health.score }}/100</span>
            </div>
            <div class="server-health-list">
              <article
                v-for="item in selectedServer.health.items"
                :key="item.key"
              >
                <GIcon
                  :name="item.icon"
                  :size="16"
                />
                <div>
                  <strong>{{ item.label }}</strong>
                  <p>{{ item.message }}</p>
                </div>
                <small>{{ item.latency ? `${item.latency}ms` : "-" }}</small>
              </article>
            </div>
          </section>

          <section
            v-if="selectedServer.logs.length"
            class="server-info-section"
          >
            <div class="server-info-section__head">
              <h3>最近错误</h3>
            </div>
            <div class="server-error-line">
              {{ selectedServer.logs[0]?.message }}
            </div>
          </section>
        </template>

        <div
          v-else
          class="server-detail-placeholder"
        >
          <GIcon
            name="servers"
            :size="34"
          />
          <span>选择一台服务器查看连接状态</span>
        </div>
      </main>
    </div>

    <Transition name="server-dialog">
      <div
        v-if="dialogVisible"
        class="server-dialog-backdrop"
        @click.self="closeDialog"
      >
        <form
          class="server-dialog"
          @submit.prevent="submitForm"
        >
          <header>
            <div>
              <p>{{ editingId ? "Edit Server" : "Add Server" }}</p>
              <h2>{{ editingId ? "编辑服务器" : "添加服务器" }}</h2>
            </div>
            <button
              type="button"
              aria-label="关闭"
              @click="closeDialog"
            >
              <GIcon
                name="close"
                :size="16"
              />
            </button>
          </header>

          <main class="server-dialog__main">
            <section class="server-form-section">
              <div class="server-helper-banner">
                <GIcon
                  name="plug-zap"
                  :size="20"
                />
                <div>
                  <strong>服务器就是 Gate 服务端所在的机器</strong>
                  <p>先把服务端程序跑起来，再把它的地址和 Token 填到这里，客户端才能创建隧道。</p>
                </div>
              </div>

              <div class="server-preset-row">
                <button
                  v-for="preset in connectionPresets"
                  :key="preset.id"
                  type="button"
                  class="server-preset"
                  @click="applyPreset(preset.id)"
                >
                  <GIcon
                    :name="preset.icon"
                    :size="16"
                  />
                  <span>{{ preset.label }}</span>
                </button>
              </div>
              <p class="server-local-note">
                <GIcon
                  name="info-circle"
                  :size="14"
                />
                本机测试会自动使用默认 Token：<code>{{ localServerToken }}</code>。正式部署请改成你自己的 Token。
              </p>
              <p class="server-mode-hint">
                <GIcon
                  :name="serverModeHint.icon"
                  :size="15"
                />
                <span>{{ serverModeHint.text }}</span>
              </p>

              <label>
                <span>名称</span>
                <input
                  v-model.trim="form.name"
                  autocomplete="off"
                  placeholder="例如：我的 VPS / 本机测试服务器"
                >
                <small>只是给你自己看的名字，不影响连接。</small>
              </label>

              <div class="server-form-grid">
                <label>
                  <span>服务器地址</span>
                  <input
                    v-model.trim="form.host"
                    autocomplete="off"
                    placeholder="公网 IP、域名或 127.0.0.1"
                    required
                  >
                  <small>服务端部署在哪台机器，就填那台机器的 IP 或域名。</small>
                </label>
                <label>
                  <span>端口</span>
                  <input
                    v-model.number="form.port"
                    type="number"
                    min="1"
                    max="65535"
                    required
                  >
                  <small>默认是服务端监听端口，通常为 7000。</small>
                </label>
              </div>

              <label>
                <span class="server-label-row">
                  Token
                  <em>服务端通行口令</em>
                </span>
                <div class="server-token-control">
                  <input
                    v-model.trim="form.token"
                    autocomplete="off"
                    :type="tokenVisible ? 'text' : 'password'"
                    placeholder="填服务端配置里的 GATE_AUTH_TOKEN"
                    required
                  >
                  <button
                    type="button"
                    :title="tokenVisible ? '隐藏 Token' : '显示 Token'"
                    @click="tokenVisible = !tokenVisible"
                  >
                    <GIcon
                      :name="tokenVisible ? 'eye-off' : 'eye'"
                      :size="15"
                    />
                  </button>
                </div>
                <small>本机测试直接用默认 Token：<code>{{ localServerToken }}</code>。正式部署时再通过 <code>GATE_AUTH_TOKEN</code> 换成你自己的长随机值。</small>
              </label>

              <div class="server-kind-picker">
                <div class="server-kind-picker__head">
                  <span>类型</span>
                  <small>只影响分类展示，不改变连接协议。</small>
                </div>
                <div class="server-kind-grid-simple">
                  <button
                    v-for="option in kindOptions"
                    :key="option.value"
                    type="button"
                    class="server-kind-card"
                    :class="{ active: form.kind === option.value }"
                    @click="selectKind(option.value)"
                  >
                    <GIcon
                      :name="option.icon"
                      :size="16"
                    />
                    <span>
                      <strong>{{ option.label }}</strong>
                      <small>{{ option.description }}</small>
                    </span>
                  </button>
                </div>
              </div>

              <div class="server-form-grid">
                <label>
                  <span>区域</span>
                  <input
                    v-model.trim="form.region"
                    autocomplete="off"
                    placeholder="可选，例如 cn-shanghai / home"
                  >
                  <small>方便以后区分多台服务器，可不填。</small>
                </label>
                <label>
                  <span>备注</span>
                  <input
                    v-model.trim="form.remark"
                    autocomplete="off"
                    placeholder="可选，例如 Docker 部署 / 家里 NAS"
                  >
                  <small>记录部署位置或维护说明。</small>
                </label>
              </div>

              <label class="server-check server-check--with-hint">
                <input
                  v-model="form.autoConnect"
                  type="checkbox"
                >
                <span>
                  启动后自动连接
                  <small>适合长期固定使用的服务器；临时测试可以先不勾。</small>
                </span>
              </label>

              <p
                v-if="formError"
                class="server-form-error"
              >
                {{ formError }}
              </p>
            </section>

            <aside class="server-guide-panel">
              <header class="server-guide-panel__header">
                <span>部署助手</span>
                <strong>先启动服务端，再添加这里的连接。</strong>
              </header>

              <details
                class="server-guide-panel__block"
                open
              >
                <summary>
                  <GIcon
                    name="monitor"
                    :size="16"
                  /> 本机测试
                </summary>
                <p>在项目根目录运行，选择“本机测试”后保存并测试连接。</p>
                <div class="server-guide-command">
                  <code>{{ localServerCommand }}</code>
                  <button
                    type="button"
                    class="server-guide-copy"
                    @click="copyLocalServerCommand"
                  >
                    <GIcon
                      name="copy"
                      :size="14"
                    />
                    复制
                  </button>
                </div>
                <div class="server-guide-kv">
                  <span>Host</span><b>127.0.0.1</b>
                  <span>Port</span><b>7000</b>
                  <span>Token</span><b>{{ localServerToken }}</b>
                </div>
                <details class="server-guide-advanced">
                  <summary>自定义端口或 Token</summary>
                  <div class="server-guide-command">
                    <code>{{ customLocalServerCommand }}</code>
                    <button
                      type="button"
                      class="server-guide-copy"
                      @click="copyGuideCommand(customLocalServerCommand, '自定义本机启动命令')"
                    >
                      <GIcon
                        name="copy"
                        :size="14"
                      />
                      复制
                    </button>
                  </div>
                </details>
              </details>

              <details
                class="server-guide-panel__block"
                open
              >
                <summary>
                  <GIcon
                    name="cloud"
                    :size="16"
                  /> 云服务器部署
                </summary>
                <p>Token 是 Gate 自己的服务端口令：<span class="server-inline-code">GATE_AUTH_TOKEN</span>，客户端必须填同一个值。</p>
                <div class="server-guide-command">
                  <span>源码启动（Linux）</span>
                  <code>{{ remoteServerCommand }}</code>
                  <button
                    type="button"
                    class="server-guide-copy"
                    @click="copyGuideCommand(remoteServerCommand, '源码启动命令')"
                  >
                    <GIcon
                      name="copy"
                      :size="14"
                    />
                    复制
                  </button>
                </div>
                <div class="server-guide-command">
                  <span>Release 二进制启动</span>
                  <code>{{ remoteBinaryCommand }}</code>
                  <button
                    type="button"
                    class="server-guide-copy"
                    @click="copyGuideCommand(remoteBinaryCommand, '二进制启动命令')"
                  >
                    <GIcon
                      name="copy"
                      :size="14"
                    />
                    复制
                  </button>
                </div>
                <div class="server-guide-command">
                  <span>Docker Compose</span>
                  <code>{{ dockerComposeCommand }}</code>
                  <button
                    type="button"
                    class="server-guide-copy"
                    @click="copyGuideCommand(dockerComposeCommand, 'Docker Compose 命令')"
                  >
                    <GIcon
                      name="copy"
                      :size="14"
                    />
                    复制
                  </button>
                </div>
                <p>Docker 只是部署方式：容器在本机运行就连 <span class="server-inline-code">127.0.0.1:5800</span>，容器在 VPS 运行就连 <span class="server-inline-code">VPS公网IP:5800</span>。</p>
              </details>

              <details class="server-guide-panel__block">
                <summary>
                  <GIcon
                    name="shield"
                    :size="16"
                  /> 端口和防火墙
                </summary>
                <ul>
                  <li><b>7000/tcp</b>：Gate 客户端连接服务端的控制端口，源码/二进制默认用它。</li>
                  <li><b>5800/tcp</b>：当前 docker-compose.yml 默认映射的服务端端口。</li>
                  <li><b>隧道端口</b>：例如你填远程端口 18080，就要开放 18080/tcp。</li>
                  <li><b>80/443</b>：只有使用域名 HTTP/HTTPS 公网入口时才需要开放。</li>
                </ul>
                <div class="server-guide-command">
                  <span>Ubuntu / Debian 防火墙示例</span>
                  <code>{{ ufwCommand }}</code>
                  <button
                    type="button"
                    class="server-guide-copy"
                    @click="copyGuideCommand(ufwCommand, 'UFW 防火墙命令')"
                  >
                    <GIcon
                      name="copy"
                      :size="14"
                    />
                    复制
                  </button>
                </div>
                <div class="server-guide-command">
                  <span>CentOS / Rocky 防火墙示例</span>
                  <code>{{ firewalldCommand }}</code>
                  <button
                    type="button"
                    class="server-guide-copy"
                    @click="copyGuideCommand(firewalldCommand, 'firewalld 防火墙命令')"
                  >
                    <GIcon
                      name="copy"
                      :size="14"
                    />
                    复制
                  </button>
                </div>
                <p>云厂商安全组也要放行同样端口；Linux 防火墙和云安全组少开一个都会连不上。</p>
              </details>

              <details class="server-guide-panel__block">
                <summary>
                  <GIcon
                    name="clipboard-list"
                    :size="16"
                  /> 表单怎么填
                </summary>
                <ul>
                  <li><b>Host</b>：你的服务器公网 IP 或域名。</li>
                  <li><b>Port</b>：源码/二进制填 7000；Docker Compose 默认填 5800。</li>
                  <li><b>Token</b>：和服务端 <span class="server-inline-code">GATE_AUTH_TOKEN</span> 完全一致。</li>
                  <li><b>类型</b>：只用于分类展示，不影响连接协议。</li>
                </ul>
              </details>

              <details class="server-guide-panel__block">
                <summary>
                  <GIcon
                    name="circle-help"
                    :size="16"
                  /> 类型怎么选
                </summary>
                <ul>
                  <li><b>Personal</b>：自己的电脑或普通 VPS。</li>
                  <li><b>Cloud</b>：阿里云、腾讯云、AWS 等云服务器。</li>
                  <li><b>NAS</b>：家里的 NAS 或小主机。</li>
                  <li><b>Company</b>：公司内网机器。</li>
                  <li><b>Docker</b>：服务端跑在容器里。</li>
                </ul>
              </details>
            </aside>
          </main>

          <footer>
            <GButton
              variant="ghost"
              @click="closeDialog"
            >
              取消
            </GButton>
            <GButton
              variant="primary"
              type="submit"
              :loading="saving"
            >
              保存
            </GButton>
          </footer>
        </form>
      </div>
    </Transition>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue"
import { useRouter } from "vue-router"
import { useFeedback } from "@composables/useFeedback"
import GButton from "@components/base/GButton.vue"
import GCard from "@components/base/GCard.vue"
import GErrorState from "@components/feedback/GErrorState.vue"
import GIcon from "@components/icons/GIcon.vue"
import { useServer } from "./composables/useServer"
import { defaultServerForm } from "./store/server"
import type { Server, ServerFormData, ServerKind, ServerStatus } from "./types"
import "./styles/server.css"

const router = useRouter()
const { toast, confirmDanger } = useFeedback()
const {
  servers,
  onlineServers,
  isLoading,
  isError,
  error,
  hasServers,
  refresh,
  retry,
  create,
  update,
  remove,
  connect,
  disconnect,
  checkHealth,
} = useServer()

const query = ref("")
const selectedId = ref<string | null>(null)
const dialogVisible = ref(false)
const editingId = ref<string | null>(null)
const saving = ref(false)
const connectingId = ref<string | null>(null)
const testingId = ref<string | null>(null)
const formError = ref("")
const tokenVisible = ref(false)

const form = reactive<ServerFormData>({ ...defaultServerForm, tags: [] })
const localServerToken = "gate-alpha-token"
const localServerCommand = "npm run dev:server"
const customLocalServerCommand =
  'npm run dev:server:local -- -Addr "127.0.0.1:7001" -Token "replace-with-a-long-random-token"'
const remoteServerCommand =
  "GATE_SERVER_ADDR=0.0.0.0:7000 GATE_AUTH_TOKEN=replace-with-a-long-random-token cargo run -p gate-server --release"
const remoteBinaryCommand =
  "GATE_SERVER_ADDR=0.0.0.0:7000 GATE_AUTH_TOKEN=replace-with-a-long-random-token ./gate-server"
const dockerComposeCommand = "GATE_AUTH_TOKEN=replace-with-a-long-random-token GATE_PORT=5800 docker compose up -d"
const ufwCommand = "sudo ufw allow 7000/tcp && sudo ufw allow 18080/tcp && sudo ufw reload"
const firewalldCommand =
  "sudo firewall-cmd --permanent --add-port=7000/tcp && sudo firewall-cmd --permanent --add-port=18080/tcp && sudo firewall-cmd --reload"

const connectionPresets = [
  {
    id: "local",
    label: "本机测试",
    icon: "monitor",
    host: "127.0.0.1",
    port: 7000,
    kind: "personal" as ServerKind,
    region: "local",
    name: "本机测试服务器",
    token: localServerToken,
  },
  {
    id: "vps",
    label: "云服务器",
    icon: "cloud",
    host: "",
    port: 7000,
    kind: "cloud" as ServerKind,
    region: "",
    name: "我的云服务器",
    token: "",
  },
  {
    id: "docker",
    label: "Docker",
    icon: "box",
    host: "127.0.0.1",
    port: 5800,
    kind: "docker" as ServerKind,
    region: "docker",
    name: "Docker Gate Server",
    token: "",
  },
]

const kindOptions: Array<{
  value: ServerKind
  label: string
  icon: string
  description: string
}> = [
  { value: "personal", label: "Personal", icon: "user", description: "个人电脑或普通 VPS" },
  { value: "cloud", label: "Cloud", icon: "cloud", description: "云服务器，有公网 IP" },
  { value: "nas", label: "NAS", icon: "hard-drive", description: "家庭 NAS 或小主机" },
  { value: "company", label: "Company", icon: "shield", description: "公司或团队服务器" },
  { value: "docker", label: "Docker", icon: "box", description: "服务端运行在容器里" },
]

const filteredServers = computed(() => {
  const keyword = query.value.toLowerCase()
  if (!keyword) return servers.value
  return servers.value.filter((server) =>
    [
      server.name,
      server.settings.host,
      String(server.settings.port),
      server.region,
      ...server.tags,
    ]
      .join(" ")
      .toLowerCase()
      .includes(keyword),
  )
})

const selectedServer = computed(() =>
  selectedId.value ? servers.value.find((server) => server.id === selectedId.value) : undefined,
)

const serverModeHint = computed(() => {
  if (form.kind === "docker") {
    return {
      icon: "box",
      text: "Docker 表示把 Gate Server 放进容器里运行，不是新的隧道类型。Docker 跑在本机就填 127.0.0.1:5800，跑在 VPS 就填 VPS 公网 IP:5800，Token 填启动容器时设置的 GATE_AUTH_TOKEN。",
    }
  }
  if (form.kind === "cloud") {
    return {
      icon: "cloud",
      text: "云服务器表示 Gate Server 跑在你的 VPS 上。Host 填公网 IP 或域名，Port 通常是 7000，Token 填服务端的 GATE_AUTH_TOKEN。",
    }
  }
  return {
    icon: "monitor",
    text: "本机测试表示 Gate Server 和桌面客户端都在这台电脑上运行，适合先把流程跑通。",
  }
})

watch(
  servers,
  (list) => {
    if (!selectedId.value || !list.some((server) => server.id === selectedId.value)) {
      selectedId.value = list[0]?.id ?? null
    }
  },
  { immediate: true },
)

onMounted(() => {
  void refresh()
})

function openCreate() {
  editingId.value = null
  Object.assign(form, { ...defaultServerForm, tags: [] })
  formError.value = ""
  tokenVisible.value = false
  dialogVisible.value = true
}

function openEdit(server: Server) {
  editingId.value = server.id
  Object.assign(form, {
    name: server.name,
    kind: server.kind,
    host: server.settings.host,
    port: server.settings.port,
    token: server.settings.token,
    region: server.region,
    remark: server.settings.remark,
    tags: [...server.tags],
    heartbeatInterval: server.settings.heartbeatInterval,
    reconnectInterval: server.settings.reconnectInterval,
    autoConnect: server.settings.autoConnect,
  })
  formError.value = ""
  tokenVisible.value = false
  dialogVisible.value = true
}

function closeDialog() {
  dialogVisible.value = false
}

async function submitForm() {
  formError.value = validateForm()
  if (formError.value) return

  saving.value = true
  try {
    if (editingId.value) {
      await update(editingId.value, { ...form, tags: [...form.tags] })
      toast.success("服务器配置已更新")
    } else {
      const created = await create({ ...form, tags: [...form.tags] })
      selectedId.value = created.id
      toast.success("服务器已保存")
    }
    closeDialog()
  } catch (err) {
    formError.value = err instanceof Error ? err.message : "保存失败"
  } finally {
    saving.value = false
  }
}

async function connectSelected() {
  if (!selectedServer.value) return
  connectingId.value = selectedServer.value.id
  try {
    await connect(selectedServer.value.id)
    toast.success("服务器已连接，可以创建 Tunnel")
  } catch (err) {
    toast.error(err instanceof Error ? err.message : "服务器连接失败")
  } finally {
    connectingId.value = null
  }
}

async function disconnectSelected() {
  if (!selectedServer.value) return
  try {
    await disconnect(selectedServer.value.id)
    toast.warning("服务器连接已断开")
  } catch (err) {
    toast.error(err instanceof Error ? err.message : "断开失败")
  }
}

async function testSelected() {
  if (!selectedServer.value) return
  testingId.value = selectedServer.value.id
  try {
    await checkHealth(selectedServer.value.id)
    toast.success("服务器连接测试通过")
  } catch (err) {
    toast.error(err instanceof Error ? err.message : "服务器连接测试失败")
  } finally {
    testingId.value = null
  }
}

function removeSelected() {
  const server = selectedServer.value
  if (!server) return
  confirmDanger({
    title: "删除服务器",
    content: `删除「${server.name}」后，本地将不再保存该服务器地址和 Token。`,
    confirmText: "删除",
    onConfirm: async () => {
      try {
        await remove(server.id)
        toast.success("服务器已删除")
      } catch (err) {
        toast.error(err instanceof Error ? err.message : "删除失败")
      }
    },
  })
}

async function copyServerInfo(server: Server) {
  const text = [
    `name=${server.name}`,
    `host=${server.settings.host}`,
    `port=${server.settings.port}`,
    `status=${server.status}`,
    `lastConnectedAt=${server.lastConnectedAt}`,
  ].join("\n")
  await navigator.clipboard.writeText(text)
  toast.success("服务器信息已复制")
}

async function copyLocalServerCommand() {
  await copyGuideCommand(localServerCommand, "本机服务端启动命令")
}

async function copyGuideCommand(text: string, label: string) {
  await navigator.clipboard.writeText(text)
  toast.success(`${label}已复制`)
}

function validateForm() {
  if (!form.host.trim()) return "请填写服务器 Host。"
  if (!form.port || form.port < 1 || form.port > 65535) return "服务器端口必须在 1-65535 之间。"
  if (!form.token.trim()) return "请填写服务端 Token。"
  return ""
}

function applyPreset(id: string) {
  const preset = connectionPresets.find((item) => item.id === id)
  if (!preset) return
  form.name = preset.name
  form.host = preset.host
  form.port = preset.port
  form.kind = preset.kind
  form.region = preset.region
  form.token = preset.token
  formError.value = ""
}

function selectKind(kind: ServerKind) {
  form.kind = kind
}

function isConnecting(id: string) {
  return connectingId.value === id
}

function statusTone(status: ServerStatus) {
  if (status === "connected") return "online"
  if (status === "connecting" || status === "reconnecting") return "warning"
  if (status === "error") return "error"
  return "offline"
}

function statusLabel(status: ServerStatus) {
  const labels: Record<ServerStatus, string> = {
    connected: "已连接",
    disconnected: "已断开",
    connecting: "连接中",
    reconnecting: "重连中",
    offline: "离线",
    maintenance: "维护中",
    error: "错误",
  }
  return labels[status]
}

function maskToken(token: string) {
  if (!token) return "-"
  if (token.length <= 8) return "********"
  return `${token.slice(0, 4)}****${token.slice(-4)}`
}
</script>

<style scoped>
.servers-page {
  width: min(100%, var(--content-max-width));
  height: 100%;
  min-height: 0;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.server-page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  flex-shrink: 0;
}

.server-page-header p,
.server-dialog header p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.server-page-header h1 {
  margin-top: 2px;
  font-size: var(--text-3xl);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.server-page-header span {
  display: block;
  margin-top: var(--space-1);
  color: var(--text-secondary);
}

.server-page-header__actions,
.server-empty__actions,
.server-detail-header__actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.server-empty,
.server-detail-placeholder,
.server-list-empty {
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-3);
  color: var(--text-tertiary);
  text-align: center;
}

.server-empty {
  min-height: 460px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-xl);
  background: var(--bg-surface);
  padding: var(--space-6);
}

.server-empty__icon {
  width: 86px;
  height: 86px;
  display: grid;
  place-items: center;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-2xl);
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.server-empty h2 {
  color: var(--text-primary);
  font-size: var(--text-2xl);
  letter-spacing: 0;
}

.server-empty p {
  max-width: 560px;
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
}

.server-shell {
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(300px, 390px) minmax(0, 1fr);
  gap: var(--space-4);
}

.server-list-panel,
.server-detail-panel,
.server-info-section {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.server-list-panel {
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.server-list-panel__toolbar {
  padding: var(--space-3);
  border-bottom: 1px solid var(--border-subtle);
}

.server-search {
  height: 36px;
  display: flex;
  align-items: center;
  gap: var(--space-2);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-tertiary);
  padding: 0 var(--space-3);
}

.server-search:focus-within {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}

.server-search input {
  min-width: 0;
  flex: 1;
  border: 0;
  outline: 0;
  background: transparent;
  color: var(--text-primary);
}

.server-list-panel__body {
  min-height: 0;
  overflow: auto;
  padding: var(--space-2);
}

.server-list-row {
  width: 100%;
  min-height: 74px;
  display: grid;
  grid-template-columns: 10px 36px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-3);
  margin-top: var(--space-1);
  padding: var(--space-3);
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--text-primary);
  text-align: left;
  cursor: pointer;
}

.server-list-row:hover,
.server-list-row.active {
  border-color: var(--border-default);
  background: var(--bg-surface-hover);
}

.server-list-row.active {
  box-shadow: inset 2px 0 0 var(--color-primary);
}

.server-list-row__status,
.server-detail-header__title > span {
  width: 9px;
  height: 9px;
  border-radius: var(--radius-full);
  background: var(--status-offline);
}

.is-online { background: var(--status-online); color: var(--status-online); }
.is-warning { background: var(--status-warning); color: var(--status-warning); }
.is-error { background: var(--status-error); color: var(--status-error); }
.is-offline { background: var(--status-offline); color: var(--status-offline); }

.server-list-row__icon {
  width: 36px;
  height: 36px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--color-primary);
}

.server-list-row__main,
.server-list-row__meta {
  min-width: 0;
}

.server-list-row__main strong,
.server-list-row__main small,
.server-list-row__meta strong,
.server-list-row__meta small {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.server-list-row__main small,
.server-list-row__meta small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.server-list-row__meta {
  max-width: 100px;
  color: var(--text-secondary);
  font-size: var(--text-xs);
  text-align: right;
}

.server-list-empty {
  min-height: 220px;
}

.server-detail-panel {
  min-width: 0;
  min-height: 0;
  overflow: auto;
  padding: var(--space-5);
}

.server-detail-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
}

.server-detail-header__title {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.server-detail-header h2 {
  font-size: var(--text-2xl);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.server-detail-header p {
  margin-top: var(--space-1);
  color: var(--text-secondary);
  font-family: var(--font-mono);
}

.server-summary-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--space-3);
  margin-top: var(--space-5);
}

.server-summary-grid article {
  min-height: 82px;
  display: grid;
  align-content: center;
  gap: var(--space-1);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.server-summary-grid span,
.server-info-list dt {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.server-summary-grid strong {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.server-info-section {
  margin-top: var(--space-4);
  padding: var(--space-4);
}

.server-info-section__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  margin-bottom: var(--space-3);
}

.server-info-section__head h3 {
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
}

.server-info-section__head button {
  width: 30px;
  height: 30px;
  display: grid;
  place-items: center;
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
}

.server-info-section__head button:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.server-info-list {
  display: grid;
  gap: var(--space-2);
}

.server-info-list div {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}

.server-info-list dd {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.server-health-list {
  display: grid;
  gap: var(--space-2);
}

.server-health-list article {
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-2);
  min-height: 46px;
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.server-health-list strong,
.server-health-list p {
  display: block;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.server-health-list p,
.server-health-list small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.server-error-line {
  border-radius: var(--radius-md);
  background: var(--color-error-muted);
  color: var(--color-error);
  padding: var(--space-3);
  overflow-wrap: anywhere;
}

.server-detail-placeholder {
  min-height: 420px;
}

.server-dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: var(--z-modal);
  display: grid;
  place-items: center;
  padding: var(--space-6);
  background: var(--color-overlay);
  backdrop-filter: blur(12px);
}

.server-dialog {
  width: min(980px, 100%);
  max-height: min(760px, calc(100vh - 48px));
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto;
  overflow: hidden;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-xl);
  background: var(--bg-surface-raised);
  box-shadow: var(--shadow-floating);
}

.server-dialog header,
.server-dialog footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-4) var(--space-5);
}

.server-dialog header {
  border-bottom: 1px solid var(--border-subtle);
}

.server-dialog header h2 {
  margin-top: 2px;
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
}

.server-dialog header button {
  width: 32px;
  height: 32px;
  display: grid;
  place-items: center;
  border: 0;
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
}

.server-dialog header button:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.server-dialog__main {
  min-height: 0;
  overflow: hidden;
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(330px, 380px);
  align-items: start;
  gap: var(--space-5);
  padding: var(--space-5);
}

.server-form-section {
  max-height: 100%;
  overflow: auto;
  display: grid;
  gap: var(--space-4);
  min-width: 0;
  padding-right: var(--space-1);
}

.server-dialog label {
  display: grid;
  gap: var(--space-2);
}

.server-dialog label span {
  color: var(--text-secondary);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
}

.server-dialog label small,
.server-kind-picker__head small,
.server-check small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  line-height: var(--leading-normal);
}

.server-label-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}

.server-label-row em {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-style: normal;
  font-weight: var(--weight-regular);
}

.server-dialog input,
.server-dialog select,
.server-dialog textarea {
  width: 100%;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-primary);
  padding: 0 var(--space-3);
  outline: 0;
}

.server-dialog input,
.server-dialog select {
  height: 38px;
}

.server-dialog textarea {
  resize: vertical;
  min-height: 84px;
  padding-block: var(--space-2);
}

.server-dialog input:focus,
.server-dialog select:focus,
.server-dialog textarea:focus {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}

.server-helper-banner {
  display: grid;
  grid-template-columns: 28px minmax(0, 1fr);
  gap: var(--space-3);
  padding: var(--space-3);
  border: 1px solid color-mix(in srgb, var(--color-primary) 28%, transparent);
  border-radius: var(--radius-md);
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.server-helper-banner strong {
  display: block;
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.server-helper-banner p {
  margin-top: 2px;
  color: var(--text-secondary);
  font-size: var(--text-sm);
  line-height: var(--leading-normal);
}

.server-preset-row {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-2);
}

.server-preset {
  min-height: 38px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-weight: var(--weight-medium);
  cursor: pointer;
}

.server-preset:hover {
  border-color: var(--color-primary);
  color: var(--text-primary);
  background: var(--bg-surface-hover);
}

.server-local-note {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  min-width: 0;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  line-height: var(--leading-normal);
}

.server-local-note code {
  color: var(--text-secondary);
  font-family: var(--font-mono);
  overflow-wrap: anywhere;
}

.server-mode-hint {
  display: grid;
  grid-template-columns: 20px minmax(0, 1fr);
  gap: var(--space-2);
  align-items: flex-start;
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-size: var(--text-xs);
  line-height: var(--leading-relaxed);
}

.server-mode-hint svg {
  margin-top: 2px;
  color: var(--color-primary);
}

.server-form-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-3);
}

.server-token-control {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 38px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.server-token-control:focus-within {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}

.server-token-control input {
  border: 0;
  box-shadow: none !important;
  background: transparent;
  font-family: var(--font-mono);
}

.server-token-control button {
  display: grid;
  place-items: center;
  border: 0;
  border-left: 1px solid var(--border-subtle);
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
}

.server-token-control button:hover {
  color: var(--text-primary);
  background: var(--bg-surface-hover);
}

.server-kind-picker {
  display: grid;
  gap: var(--space-2);
}

.server-kind-picker__head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-3);
  color: var(--text-secondary);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
}

.server-kind-grid-simple {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-2);
}

.server-kind-card {
  min-height: 62px;
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr);
  align-items: center;
  gap: var(--space-2);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  padding: var(--space-2) var(--space-3);
  text-align: left;
  cursor: pointer;
}

.server-kind-card:hover,
.server-kind-card.active {
  border-color: var(--color-primary);
  background: var(--color-primary-muted);
  color: var(--text-primary);
}

.server-kind-card strong,
.server-kind-card small {
  display: block;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.server-kind-card strong {
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.server-kind-card small {
  margin-top: 2px;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.server-check {
  display: inline-flex !important;
  grid-template-columns: auto minmax(0, 1fr);
  align-items: flex-start;
  gap: var(--space-2) !important;
}

.server-check input {
  width: 16px;
  height: 16px;
  accent-color: var(--color-primary);
}

.server-check--with-hint > span {
  display: grid;
  gap: 2px;
}

.server-form-error {
  border-radius: var(--radius-md);
  background: var(--color-error-muted);
  color: var(--color-error);
  padding: var(--space-3);
}

.server-guide-panel {
  max-height: 100%;
  overflow: auto;
  display: grid;
  align-content: start;
  gap: var(--space-2);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  background: color-mix(in srgb, var(--bg-input) 72%, transparent);
}

.server-guide-panel__header {
  display: grid;
  gap: 2px;
  padding: var(--space-1) var(--space-1) var(--space-2);
}

.server-guide-panel__header span {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.server-guide-panel__header strong {
  color: var(--text-primary);
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
}

.server-guide-panel__block {
  padding: 0;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  overflow: hidden;
}

.server-guide-panel__block[open] {
  padding-bottom: var(--space-3);
}

.server-guide-panel__block summary {
  display: flex;
  align-items: center;
  min-height: 42px;
  gap: var(--space-2);
  padding: 0 var(--space-3);
  color: var(--text-primary);
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
  cursor: pointer;
  list-style: none;
}

.server-guide-panel__block summary::-webkit-details-marker {
  display: none;
}

.server-guide-panel__block summary::after {
  content: "";
  width: 7px;
  height: 7px;
  margin-left: auto;
  border-right: 1.5px solid currentColor;
  border-bottom: 1.5px solid currentColor;
  opacity: 0.65;
  transform: rotate(45deg) translateY(-2px);
  transition: transform var(--duration-fast) var(--ease-out);
}

.server-guide-panel__block[open] summary::after {
  transform: rotate(225deg) translateY(-1px);
}

.server-guide-panel__block p,
.server-guide-panel__block li {
  color: var(--text-secondary);
  font-size: var(--text-xs);
  line-height: var(--leading-relaxed);
}

.server-guide-panel__block p,
.server-guide-panel__block ul,
.server-guide-panel__block ol,
.server-guide-panel__block .server-guide-command,
.server-guide-panel__block .server-guide-kv,
.server-guide-panel__block .server-guide-advanced {
  margin-inline: var(--space-3);
}

.server-guide-panel__block code {
  display: block;
  overflow: auto;
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  padding: var(--space-2);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.server-guide-command {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: var(--space-2);
  align-items: end;
}

.server-guide-command + .server-guide-command {
  margin-top: var(--space-2);
}

.server-guide-command > span {
  grid-column: 1 / -1;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
}

.server-guide-kv {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: var(--space-2) var(--space-3);
  padding: var(--space-2);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
}

.server-guide-kv span {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.server-guide-kv b {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.server-inline-code,
.server-dialog label small code {
  border-radius: var(--radius-sm);
  background: var(--bg-surface);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  padding: 1px 4px;
}

.server-guide-copy {
  width: 70px;
  min-height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-1);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  padding: 0 var(--space-2);
  font-size: var(--text-xs);
  cursor: pointer;
}

.server-guide-copy:hover {
  border-color: var(--color-primary);
  color: var(--text-primary);
}

.server-guide-advanced {
  display: grid;
  gap: var(--space-2);
}

.server-guide-advanced summary {
  width: fit-content;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  cursor: pointer;
}

.server-guide-advanced summary:hover {
  color: var(--text-secondary);
}

.server-guide-panel__block ol,
.server-guide-panel__block ul {
  display: grid;
  gap: var(--space-1);
  padding-left: var(--space-4);
}

.server-dialog footer {
  justify-content: flex-end;
  border-top: 1px solid var(--border-subtle);
}

.server-dialog-enter-active,
.server-dialog-leave-active {
  transition: opacity var(--duration-base) var(--ease-out);
}

.server-dialog-enter-from,
.server-dialog-leave-to {
  opacity: 0;
}

@media (max-width: 1120px) {
  .server-shell {
    grid-template-columns: 1fr;
  }

  .server-list-panel {
    max-height: 360px;
  }

  .server-summary-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 760px) {
  .server-page-header,
  .server-detail-header {
    flex-direction: column;
  }

  .server-page-header__actions,
  .server-detail-header__actions {
    width: 100%;
    align-items: stretch;
    flex-direction: column;
  }

  .server-summary-grid,
  .server-form-grid,
  .server-preset-row,
  .server-kind-grid-simple,
  .server-dialog__main {
    grid-template-columns: 1fr;
  }

  .server-dialog__main {
    overflow: auto;
  }

  .server-form-section,
  .server-guide-panel {
    max-height: none;
    overflow: visible;
  }

  .server-guide-panel {
    padding: var(--space-2);
  }
}
</style>
