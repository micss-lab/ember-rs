#! /usr/bin/env sh

end=$((SECONDS+60))
count=0
while [ $SECONDS -lt $end ]; do
  curl -s -o /dev/null http://"$1"/
  echo -n -e "\rRequest count: $count"
  count=$((count+1))
done

echo
echo "Total sequential requests in 1 minute: $count"
