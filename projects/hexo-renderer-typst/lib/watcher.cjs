
const fs = require('fs');

const WATCH_EVENT = {
  CREATED: 1,
  REMOVED: 2,
  MODIFIED: 3,
}

const WATCH_MODE = {
  LAST_MODIFIED: 1,
  ALL_FILES: 2,
}

class Watcher {
  constructor(hexo, compiler) {
    this.hexo = hexo;
    this.compiler = compiler;
    this.lastUpdates = [];
    this.lifetime = 0;
    this.lastUpdated = [this.lifetime-1, undefined];
    this.watchState = new Map();
    this.watchMode = WATCH_MODE.LAST_MODIFIED;
  }

  watch(state) {
    // console.log('[typst] watcher watch post', state.title);
    if (state.callback !== undefined) {
      console.error('watcher callback is not undefined', state.title);
      this.unwatch(state);
    }

    const callback = () => {
      if (!fs.existsSync(state.full_source)) {
        return this.processPost(WATCH_EVENT.REMOVED, state);
      }

      return this.processPost(WATCH_EVENT.MODIFIED, state);
    }
  
    state.callback = callback;
    this.compiler.watch(state.full_source, callback);
  }

  unwatch(state) {
    // console.log('[typst] watcher unwatch post', state.title);
    if (state.callback === undefined) {
      console.error('watcher callback is not registered', state.title);
      return;
    }
    this.compiler.unwatch(state.full_source, state.callback);
  }

  processPost(event, state) {
    // console.log('[typst] watcher process post', event, state.lifetime, state.title, state.updated, state.full_source);
    console.log('[typst] watching', `<${state.title}>`, 'of', state.full_source, 'updated at', state.updated.format('YYYY-MM-DD HH:mm:ss.SSS'));

    switch (event) {
    case WATCH_EVENT.CREATED:
      if (this.watchMode === WATCH_MODE.ALL_FILES) {
        return this.watch(state);
      }
      return 
    case WATCH_EVENT.REMOVED:
      if (state.callback !== undefined) {
        this.unwatch(state);
      }
      this.watchState.delete(state.full_source);
      return;
    case WATCH_EVENT.MODIFIED:
      // todo: regular way to trigger post render
      return this.hexo.post.render(post.full_source, state.post).then(() => post.save());
    }
  }

  notifyPost(post, lifetime) {
    let state = this.watchState.get(post.full_source);
    if (!state) {
      state = {
        post,
        title: post.title,
        updated: post.updated,
        source: post.source,
        full_source: post.full_source,
        lifetime,
      }
      this.watchState.set(post.full_source, state);
      this.processPost(WATCH_EVENT.CREATED, state);
    } else if (state.lifetime < lifetime) {
      state.lifetime = lifetime;
      state.post = post;
    }

    const shouldUpdate = 
      (this.lastUpdated[0] < lifetime) ||
      (this.lastUpdated[1] === undefined) ||
      (this.lastUpdated[1].updated < post.updated);
    if (shouldUpdate) {
      this.lastUpdated = [lifetime, state];
    }
  }
  
  startWatch(data) {
    const lifetime = this.lifetime++;
    if (this.lastUpdated[1]) {
      this.lastUpdates.push(this.lastUpdated[1]);
    }
    this.lastUpdated = [lifetime, undefined];
    const watchPosts = model => {
      const posts = model.toArray();
  
      return Promise.all(posts.map(post => {
        post.content = post._content;
        post.site = {data};

        return this.notifyPost(post, lifetime);
      })).then(() => {
        if (this.watchMode === WATCH_MODE.LAST_MODIFIED) {
          const [lastUpdatedLifetime, state] = this.lastUpdated;
          if (lastUpdatedLifetime === lifetime) {
            const allLastUpdates = this.lastUpdates.
              splice(0, this.lastUpdates.length).
              filter(lastState => (lastState.full_source !== state.full_source && 
                lastState.callback !== undefined));
            for (const lastState of allLastUpdates) {
              this.unwatch(lastState);
            }

            if (state.callback === undefined) {
              this.watch(state);
            }
          }
        }
      });
    };
  
    return Promise.all([
      watchPosts(this.hexo.model('Post')),
      // todo: should we watch pages?
      // watchPosts(this.model('Page'))
    ]);
  }
}

module.exports = Watcher;
