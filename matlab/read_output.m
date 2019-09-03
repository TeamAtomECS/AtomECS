function output = read_output(file)

fh = fopen(file, 'r');
oc = onCleanup(@() fclose(fh));

output = {};
while ~feof(fh)
    % example header: step 0, 1
    header = fgetl(fh);
    header_data = sscanf(header, 'step %d, %d');
    step = header_data(1);
    natoms = header_data(2);
    % example atom entry: 1,2: 0.0,0.0,-0.009500550415211395
    data = textscan(fh, '%d,%d: (%f,%f,%f)\n', natoms);
    gen = cat(1,data{:,1});
    id = cat(1,data{:,2});
    x = cat(1, data{:,3});
    y = cat(1, data{:,4});
    z = cat(1, data{:,5});
    output{end+1} = struct('gen', gen, 'id', id, 'vec', cat(2, x, y, z));
end
output = cat(1, output{:});

end

