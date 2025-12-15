import DefaultTheme from 'vitepress/theme'
import './custom.css'

import FeatureCard from './components/FeatureCard.vue'
import FeatureGrid from './components/FeatureGrid.vue'
import CategoryBadge from './components/CategoryBadge.vue'
import TipBox from './components/TipBox.vue'
import HomeFeatures from './components/HomeFeatures.vue'

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component('FeatureCard', FeatureCard)
    app.component('FeatureGrid', FeatureGrid)
    app.component('CategoryBadge', CategoryBadge)
    app.component('TipBox', TipBox)
    app.component('HomeFeatures', HomeFeatures)
  }
}
