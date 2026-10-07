module.exports = {
      darkMode: 'class',
      theme: {
        extend: {
          colors: {
            graphite: {
              950: '#0B0C0E',
              900: '#111215',
              850: '#16181C',
              800: '#1C1E23',
              750: '#23262D',
              700: '#2B2F38',
              600: '#3D424E',
              500: '#5A6172',
              400: '#8A93A6',
              300: '#B0B7C6',
              200: '#D5DAE3',
              100: '#EDF0F5'
            },
            accent: {
              50: '#F5F3FF',
              400: '#A78BFA',
              500: '#8B5CF6',
              600: '#7C3AED',
              700: '#6D28D9'
            },
            meter: {
              green: '#10B981',
              yellow: '#F59E0B',
              red: '#EF4444'
            }
          },
          fontFamily: {
            sans: ['-apple-system', 'BlinkMacSystemFont', '"Segoe UI"', 'Roboto', 'Inter', 'Helvetica', 'Arial', 'sans-serif'],
            mono: ['"SF Mono"', 'ui-monospace', 'Menlo', 'Monaco', 'Consolas', 'monospace']
          },
          animation: {
            'pulse-subtle': 'pulseSubtle 2.5s cubic-bezier(0.4, 0, 0.6, 1) infinite',
          },
          keyframes: {
            pulseSubtle: {
              '0%, 100%': { opacity: 1 },
              '50%': { opacity: 0.75 },
            }
          }
        }
      }
    };
module.exports.content = {relative:true,files:["./ui/index.html","./ui/app.js"]};
