import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView,
    },
    {
      path: '/title',
      name: 'title',
      component: () => import('../views/TitleScreenView.vue'),
    },
    {
      path: '/game',
      name: 'game',
      component: () => import('../views/GameView.vue'),
    },
    {
      path: '/chat',
      name: 'chat',
      component: () => import('../views/ChatView.vue'),
    },
    {
      path: '/editor',
      name: 'editor',
      component: () => import('../views/WorkflowEditor.vue'),
    },
    {
      path: '/character-editor',
      name: 'character-editor',
      component: () => import('../views/CharacterEditorView.vue'),
    },
    {
      path: '/assets',
      name: 'assets',
      component: () => import('../views/SceneAssetsView.vue'),
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('../views/SettingsView.vue'),
    },
    {
      path: '/characters',
      name: 'characters',
      component: () => import('../views/CharacterGalleryView.vue'),
    },
    {
      path: '/group-chat',
      name: 'group-chat',
      component: () => import('../views/GroupChatView.vue'),
    },
    {
      path: '/analytics',
      name: 'analytics',
      component: () => import('../views/AnalyticsView.vue'),
    },
    {
      path: '/quality',
      name: 'quality',
      component: () => import('../views/QualitySuiteView.vue'),
    },
    {
      path: '/marketplace',
      name: 'marketplace',
      component: () => import('../views/MarketplaceView.vue'),
    },
    {
      path: '/plugins',
      name: 'plugins',
      component: () => import('../views/PluginView.vue'),
    },
    {
      path: '/audio',
      name: 'audio',
      component: () => import('../views/AudioView.vue'),
    },
    {
      path: '/knowledge',
      name: 'knowledge',
      component: () => import('../views/KnowledgeBaseView.vue'),
    },
    {
      path: '/dialogue-editor',
      name: 'dialogue-editor',
      component: () => import('../views/DialogueEditorView.vue'),
    },
    {
      path: '/story-events',
      name: 'story-events',
      component: () => import('../views/StoryEventEditorView.vue'),
    },
    {
      path: '/endings',
      name: 'endings',
      component: () => import('../views/EndingEditorView.vue'),
    },
    {
      path: '/scene-editor',
      name: 'scene-editor',
      component: () => import('../views/SceneEditorView.vue'),
    },
    {
      path: '/cg-gallery',
      name: 'cg-gallery',
      component: () => import('../views/CGGalleryView.vue'),
    },
    {
      path: '/backlog',
      name: 'backlog',
      component: () => import('../views/BacklogView.vue'),
    },
  ],
})

export default router
