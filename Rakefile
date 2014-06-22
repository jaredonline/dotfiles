require 'pathname'

EXCLUDED_FILES = [
  '.',
  '..',
  '.git'
]

task :bootstrap do
  home_dir    = Pathname.new(File.expand_path("~"))
  current_dir = Pathname.new(File.expand_path(__FILE__)).parent

  puts current_dir.children.inspect

  current_dir.children.each do |path|
    basename = path.basename
    if basename.to_s[0] == "." && !EXCLUDED_FILES.include?(basename.to_s)
      sym_link = home_dir.join(basename)

      if sym_link.exist?
        sym_link.delete
      end
      sym_link.make_symlink(path)
    end
  end
end
