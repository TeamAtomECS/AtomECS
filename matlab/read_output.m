function output = read_output(file, varargin)

ip = inputParser();
ip.addParameter('Format', '(%f, %f, %f)');
if nargin > 1
    ip.parse(varargin{:});
else
    ip.parse();
end
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
    data = textscan(fh, ['%d,%d: ' ip.Results.Format ' \n'], natoms);
    gen = cat(1,data{:,1});
    id = cat(1,data{:,2});
    x = cat(2, data{:,3:end});
    output{end+1} = struct('gen', gen, 'id', id, 'vec', x);
end
output = cat(1, output{:});

end

