function output = read_output(file)

fh = fopen(file, 'r');
oc = onCleanup(@() fclose(fh));

output = {};
while ~feof(fh)
   natoms = str2num(fgetl(fh));
   pos = textscan(fh, '%f,%f,%f\n', natoms);
   pos = cat(2, pos{:})';
   output{end+1} = pos;
end

end

