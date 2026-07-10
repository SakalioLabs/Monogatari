const translations = {
  en: {
    pageTitle: 'Monogatari Offline',
    status: 'Offline mode',
    title: 'Monogatari is offline',
    description: 'The workbench shell is cached, but this route needs a connection or a previously cached response.',
    returnLabel: 'Return to the workbench',
  },
  'zh-CN': {
    pageTitle: 'Monogatari 离线',
    status: '离线模式',
    title: 'Monogatari 当前处于离线状态',
    description: '工作区外壳已缓存，但此页面需要网络连接或此前缓存的响应。',
    returnLabel: '返回工作区',
  },
  'ja-JP': {
    pageTitle: 'Monogatari オフライン',
    status: 'オフラインモード',
    title: 'Monogatari はオフラインです',
    description: 'ワークスペースはキャッシュされていますが、このページには接続または以前のキャッシュが必要です。',
    returnLabel: 'ワークスペースに戻る',
  },
  'ko-KR': {
    pageTitle: 'Monogatari 오프라인',
    status: '오프라인 모드',
    title: 'Monogatari가 오프라인 상태입니다',
    description: '워크스페이스 셸은 캐시되어 있지만 이 페이지에는 연결 또는 이전에 캐시된 응답이 필요합니다.',
    returnLabel: '워크스페이스로 돌아가기',
  },
}

let locale = 'en'
try {
  const stored = localStorage.getItem('monogatari-locale')
  if (stored && translations[stored]) locale = stored
  else if (navigator.language.toLowerCase().startsWith('zh')) locale = 'zh-CN'
  else if (navigator.language.toLowerCase().startsWith('ja')) locale = 'ja-JP'
  else if (navigator.language.toLowerCase().startsWith('ko')) locale = 'ko-KR'
} catch {
  locale = 'en'
}

const copy = translations[locale]
document.documentElement.lang = locale
document.title = copy.pageTitle
document.getElementById('status').textContent = copy.status
document.getElementById('title').textContent = copy.title
document.getElementById('description').textContent = copy.description
document.getElementById('return-label').textContent = copy.returnLabel
