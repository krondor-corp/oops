module Jekyll
  class RawDocPage < Page
    def initialize(site, base, dir, name, content)
      @site = site
      @base = base
      @dir  = dir
      @name = name

      self.data = {}
      self.content = content
      self.process(name)
    end

    def render_with_liquid?
      false
    end
  end

  class RawDocsGenerator < Generator
    safe true
    priority :low

    def generate(site)
      docs = site.collections["docs"]&.docs
      return unless docs

      nav = site.data["nav"] || []

      docs.each do |doc|
        slug = doc.data["slug"] || doc.basename_without_ext
        page = RawDocPage.new(site, site.source, "docs/#{slug}", "raw.txt", doc.content)
        site.pages << page
      end

      docs_by_slug = {}
      docs.each { |d| docs_by_slug[d.data["slug"] || d.basename_without_ext] = d }

      ordered_docs = []
      nav.each do |section|
        (section["items"] || []).each do |slug|
          ordered_docs << docs_by_slug[slug] if docs_by_slug[slug]
        end
      end
      remaining = docs - ordered_docs
      ordered_docs.concat(remaining)

      full_parts = ordered_docs.map do |doc|
        title = doc.data["title"] || doc.basename_without_ext
        "# #{title}\n\n#{doc.content.strip}"
      end

      full_page = RawDocPage.new(site, site.source, "", "llms-full.txt", full_parts.join("\n\n---\n\n"))
      site.pages << full_page
    end
  end
end
